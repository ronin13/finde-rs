use crate::constants::{CHAN_TIMEOUT_S, INDEX_HEAP_SIZE, INIT_THREADS};
use crate::haslen::HasLen;

use crate::indexer;
use crate::resource::Resource;
use crate::scheduler;
use anyhow::{anyhow, Context, Result};
use crossbeam::channel::select;
use log::{debug, info, trace};

use crate::resource::Response;
use crossbeam::channel::unbounded;
use crossbeam::channel::{Receiver, Sender};
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use threadpool::ThreadPool;

use std::marker::{Send, Sync};

impl<T: Send + 'static> HasLen for Receiver<T> {
    fn len(&self) -> usize {
        self.len()
    }
}

// #[derive(Debug)]
pub struct Crawler<T: FromStr + Send + Sync + std::fmt::Debug + 'static> {
    pool: ThreadPool,
    crawler_chan: Sender<T>,
    processor_chan: Receiver<T>,
    resource: Arc<Box<dyn Resource<T>>>,
    initial_threads: usize,
    max_threads: usize,
    index_dir: String,
    indexer_heap_size: usize,
}

impl<T: FromStr + Send + Sync + std::fmt::Debug + 'static> Crawler<T> {
    pub fn new(
        _resource: Box<dyn Resource<T>>,
        _initial_threads: Option<usize>,
        _max_threads: Option<usize>,
        _index_dir: String,
        _indexer_heap_size: Option<usize>,
    ) -> Crawler<T> {
        let (_crawler_chan, _processor_chan): (Sender<T>, Receiver<T>) = unbounded();
        let ival = _initial_threads.unwrap_or(INIT_THREADS);
        Crawler {
            pool: ThreadPool::new(ival),
            crawler_chan: _crawler_chan,
            processor_chan: _processor_chan,
            resource: Arc::new(_resource),
            initial_threads: ival,
            max_threads: _max_threads.unwrap_or_else(num_cpus::get),
            index_dir: _index_dir,
            indexer_heap_size: _indexer_heap_size.unwrap_or(INDEX_HEAP_SIZE),
        }
    }

    fn run_indexer(
        index_dir: String,
        heap_size: usize,
    ) -> (thread::JoinHandle<Result<()>>, Sender<String>) {
        let (file_chan, index_chan): (Sender<String>, Receiver<String>) = unbounded();
        (
            thread::spawn(move || -> Result<()> {
                match indexer::build_index(index_chan, index_dir, heap_size) {
                    Ok(_x) => Ok(()),
                    Err(e) => Err(anyhow!("Indexer failed due to {:?}", e)),
                }
            }),
            file_chan,
        )
    }

    fn run_scheduler(&self) -> thread::JoinHandle<Result<()>> {
        let pool_t = self.pool.clone();
        let processor_t = self.processor_chan.clone();
        scheduler::run(pool_t, processor_t, self.initial_threads, self.max_threads)
    }

    fn recv_from_channel_with_timeout(receiver: &Receiver<T>, timeout: u64) -> Result<Option<T>> {
        select! {
            recv(receiver) -> msg => {
                let message = msg?;
                Ok(Some(message))

            },
            default(Duration::from_secs(timeout)) => Ok(None),
        }
    }

    fn crawl_this(
        sender: Sender<T>,
        receiver: Receiver<T>,
        result: Sender<String>,
        resource: Arc<Box<dyn Resource<T>>>,
    ) -> Result<()> {
        let mut root: T;
        let whoami: String = format!("{:?}", thread::current().id());
        debug!("Crawling in thread {}", whoami);

        let mut response: Response<T>;

        loop {
            let _root = Crawler::recv_from_channel_with_timeout(&receiver, CHAN_TIMEOUT_S)?;

            root = match _root {
                Some(x) => x,
                None => {
                    info!("Crawling done in {}, leaving, bye!", whoami);
                    return Ok(());
                }
            };

            response = resource.get_dirs_and_leaves(&root);
            match response {
                Response::DirFileResponse { dirs, files } => {
                    for dir in dirs.into_iter() {
                        sender.send(dir)?;
                    }
                    for fil in files.into_iter() {
                        result.send(fil)?;
                    }
                } // _ => return Err(anyhow!("Unsupported responses received {:?}", response)),
            }

            trace!("{:?} crawling {:?}", whoami, root);
        }
    }

    pub fn run(&self) -> Result<()> {
        let path = self.resource.get_path()?;

        self.crawler_chan.send(path)?;

        let (indexer_thread, file_chan) =
            Crawler::<T>::run_indexer(self.index_dir.clone(), self.indexer_heap_size);

        for _ in 1..=self.initial_threads {
            let crawler = self.crawler_chan.clone();
            let processor = self.processor_chan.clone();
            let results = file_chan.clone();
            let resource = self.resource.clone();
            self.pool.execute(move || {
                Crawler::crawl_this(crawler, processor, results, resource)
                    .expect("Thread creation failed");
            });
        }

        let scheduler_thread = self.run_scheduler();

        info!("Waiting on upto {} crawler threads", self.max_threads);
        self.pool.join();
        drop(file_chan);

        scheduler_thread
            .join()
            .unwrap()
            .context("Scheduler execution failed")?;

        indexer_thread
            .join()
            .unwrap()
            .context("Indexer thread failed")?;

        if !self.processor_chan.is_empty() {
            return Err(anyhow!(
                "Failed to crawl everything since processor chan is still non empty"
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use crate::crawler::Crawler;
    use crossbeam::channel::{unbounded, Receiver, Sender};
    use std::path::PathBuf;

    #[should_panic]
    #[test]
    fn test_root_from_disconnected_channel() {
        let (_, empty_receiver): (Sender<PathBuf>, Receiver<PathBuf>) = unbounded();

        let _ =
            Crawler::recv_from_channel_with_timeout(&empty_receiver, 0).expect("Failed to read");
        ()
    }

    #[test]
    fn test_root_from_channel() {
        let (sender, receiver): (Sender<PathBuf>, Receiver<PathBuf>) = unbounded();
        let _ = sender.send(PathBuf::from("TESTM"));

        let mut root_path = Crawler::recv_from_channel_with_timeout(&receiver, 100);
        assert_eq!(root_path.unwrap(), Some(PathBuf::from("TESTM".to_string())));

        root_path = Crawler::recv_from_channel_with_timeout(&receiver, 1);
        assert_eq!(root_path.unwrap(), None)
    }
}
