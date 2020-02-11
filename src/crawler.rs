use crate::constants::{CHAN_TIMEOUT_S, INIT_THREADS, MAX_THREADS};
use crate::indexer;
use crate::scheduler;
use anyhow::{anyhow, Context, Result};
use crossbeam::channel::select;
// use crossbeam::channel::Select;
use crate::haslen::HasLen;
use crossbeam::channel::unbounded;
use crossbeam::channel::{Receiver, Sender};
use log::{debug, info, trace, warn};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use threadpool::ThreadPool;
use walkdir::WalkDir;

// trait Crawler<T: FromStr> {
// fn crawl_this(&self, sender: Sender<T>, receiver: Receiver<T>, result: Sender<String>) -> Result<()>;
// }

impl HasLen for Receiver<PathBuf> {
    fn len(&self) -> usize {
        self.len()
    }
}

#[derive(Debug)]
pub struct FileCrawler {
    root: String,
    pool: ThreadPool,
    crawler_chan: Sender<PathBuf>,
    processor_chan: Receiver<PathBuf>,
}

impl FileCrawler {
    pub fn new(_root: String) -> FileCrawler {
        let (_crawler_chan, _processor_chan): (Sender<PathBuf>, Receiver<PathBuf>) = unbounded();
        FileCrawler {
            root: _root,
            pool: ThreadPool::new(INIT_THREADS),
            crawler_chan: _crawler_chan,
            processor_chan: _processor_chan,
        }
    }

    fn run_indexer() -> (thread::JoinHandle<()>, Sender<String>) {
        let (file_chan, index_chan): (Sender<String>, Receiver<String>) = unbounded();
        (
            thread::spawn(move || {
                indexer::build_index(index_chan).unwrap();
            }),
            file_chan,
        )
    }

    fn run_scheduler(&self) -> thread::JoinHandle<()> {
        let pool_t = self.pool.clone();
        let processor_t = self.processor_chan.clone();
        scheduler::run(pool_t, processor_t)
    }

    pub fn run(&self) -> Result<()> {
        // Initial seed.
        self.crawler_chan
            .send(PathBuf::from(self.root.clone()))
            .context("Failed to send root path")?;

        let (indexer_thread, file_chan) = FileCrawler::run_indexer();

        for _ in 1..=INIT_THREADS {
            let crawler = self.crawler_chan.clone();
            let processor = self.processor_chan.clone();
            let results = file_chan.clone();
            self.pool.execute(move || {
                FileCrawler::crawl_this(crawler, processor, results)
                    .expect("Thread creation failed");
            });
        }

        let scheduler_thread = self.run_scheduler();

        info!("Waiting on upto {} crawler threads", MAX_THREADS);
        self.pool.join();
        drop(file_chan);
        scheduler_thread
            .join()
            .expect("Runtime issue with scheduler thread join");

        indexer_thread
            .join()
            .expect("Runtime issue while waiting on indexer thread");

        if !self.processor_chan.is_empty() {
            return Err(anyhow!(
                "Failed to crawl everything since processor chan is still non empty"
            ));
        }
        Ok(())
    }

    fn pathbuf_to_string(pbuf: PathBuf) -> Result<String> {
        let path_to_str = pbuf.to_str();
        match path_to_str {
            Some(pstr) => Ok(pstr.to_string()),
            None => Err(anyhow!("Failed to decode to String")),
        }
    }

    fn root_from_channel(receiver: &Receiver<PathBuf>, timeout: u64) -> Result<Option<String>> {
        // let mut sel = Select::new();
        // let _ = sel.recv(receiver);
        // let oper = sel.select_timeout(Duration::from_secs(timeout));
        // match oper {
        //     Err(_) => Ok(String::new()),
        //     Ok(oper) => {
        //         let message = oper.recv(receiver)?;
        //         FileCrawler::pathbuf_to_string(message)
        //     }
        // }

        select! {
            recv(receiver) -> msg => {
                let message = msg?;
                let res = FileCrawler::pathbuf_to_string(message)?;
                Ok(Some(res))

            },
            default(Duration::from_secs(timeout)) => Ok(None),
        }
    }

    /// Reads from receiver channel the paths to crawl and
    /// -> sends directory paths to sender channel.
    /// -> sends file paths to indexer on result channel
    /// Runs in a threadpool.
    fn crawl_this(
        sender: Sender<PathBuf>,
        receiver: Receiver<PathBuf>,
        result: Sender<String>,
    ) -> Result<()> {
        let mut root: String;
        let whoami: String = format!("{:?}", thread::current().id());
        debug!("Crawling in thread {}", whoami);

        loop {
            let _root = FileCrawler::root_from_channel(&receiver, CHAN_TIMEOUT_S)?;
            if _root.is_none() {
                info!("Crawling done in {}, leaving, bye!", whoami);
                return Ok(());
            }
            root = _root.unwrap();
            let mut filevec: Vec<String> = vec![];

            trace!("{} crawling {}", whoami, root);
            for entry in WalkDir::new(&root).max_depth(1).into_iter().skip(1) {
                match entry {
                    Ok(dirent) => match dirent.metadata() {
                        Ok(metadata) => {
                            if metadata.is_dir() {
                                sender
                                    .send(dirent.path().to_path_buf().to_owned())
                                    .expect("Failed to send. Boo!");
                            } else {
                                filevec.push(dirent.path().to_str().unwrap().to_string());
                            }
                        }
                        Err(e) => warn!("Ignoring due to error {}", e),
                    },
                    Err(err) => warn!("Ignoring entry due to {}", err),
                }
            }
            for fil in filevec.into_iter() {
                // info!("Sending {:?} to channel", fil);
                result.send(fil)?;
            }
        }
    }
}

#[cfg(test)]
mod test {

    use crate::crawler::FileCrawler;
    use crossbeam::channel::{unbounded, Receiver, Sender};
    use std::path::PathBuf;

    #[should_panic]
    #[test]
    fn test_root_from_disconnected_channel() {
        let (_, empty_receiver): (Sender<PathBuf>, Receiver<PathBuf>) = unbounded();

        let _ = FileCrawler::root_from_channel(&empty_receiver, 0).expect("Failed to read");
        ()
    }

    #[test]
    fn test_root_from_channel() {
        let (sender, receiver): (Sender<PathBuf>, Receiver<PathBuf>) = unbounded();
        let _ = sender.send(PathBuf::from("TESTM"));

        let mut root_path = FileCrawler::root_from_channel(&receiver, 100);
        assert_eq!(root_path.unwrap(), Some("TESTM".to_string()));

        root_path = FileCrawler::root_from_channel(&receiver, 1);
        assert_eq!(root_path.unwrap(), None)
    }
}
