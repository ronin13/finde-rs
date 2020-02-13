use crate::constants::{INIT_THREADS, MAX_THREADS};
use crate::indexer;
use crate::resource::Resource;
use crate::scheduler;
// use anyhow::{anyhow, Context, Result};
use anyhow::{anyhow, Result};
// use crossbeam::channel::Select;
use crate::haslen::HasLen;
use crossbeam::channel::unbounded;
use crossbeam::channel::{Receiver, Sender};
// use log::{info, trace, warn};
use log::info;
use std::str::FromStr;
use std::thread;
use threadpool::ThreadPool;
// use std::marker::Sized;
use std::sync::Arc;

use std::marker::{Send, Sync};
// use std::string::FromStr;

impl<T: Send + 'static> HasLen for Receiver<T> {
    fn len(&self) -> usize {
        self.len()
    }
}

// #[derive(Debug)]
pub struct Crawler<T: FromStr + Send + Sync + 'static> {
    pool: ThreadPool,
    crawler_chan: Sender<T>,
    processor_chan: Receiver<T>,
    resource: Arc<Box<dyn Resource<T>>>,
}

impl<T: FromStr + Send + Sync + 'static> Crawler<T> {
    pub fn new(_resource: Box<dyn Resource<T>>) -> Crawler<T> {
        let (_crawler_chan, _processor_chan): (Sender<T>, Receiver<T>) = unbounded();
        Crawler {
            pool: ThreadPool::new(INIT_THREADS),
            crawler_chan: _crawler_chan,
            processor_chan: _processor_chan,
            resource: Arc::new(_resource),
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
        // self.crawler_chan
        // .send(_tmp.to_resource().unwrap())
        // .context("Failed to send root path")?;

        self.crawler_chan
            .send(self.resource.to_resource().unwrap())?;
        //  self.crawler_chan.send(self.resource.clone())?;

        let (indexer_thread, file_chan) = Crawler::<T>::run_indexer();

        for _ in 1..=INIT_THREADS {
            let crawler = self.crawler_chan.clone();
            let processor = self.processor_chan.clone();
            let results = file_chan.clone();
            let resource = self.resource.clone();
            self.pool.execute(move || {
                resource
                    .crawl_this(crawler, processor, results)
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
}

// #[cfg(test)]
// mod test {

//     use crate::crawler::Crawler;
//     use crossbeam::channel::{unbounded, Receiver, Sender};
//     use std::path::PathBuf;

//     #[should_panic]
//     #[test]
//     fn test_root_from_disconnected_channel() {
//         let (_, empty_receiver): (Sender<PathBuf>, Receiver<PathBuf>) = unbounded();

//         let _ = Crawler::root_from_channel(&empty_receiver, 0).expect("Failed to read");
//         ()
//     }

//     #[test]
//     fn test_root_from_channel() {
//         let (sender, receiver): (Sender<PathBuf>, Receiver<PathBuf>) = unbounded();
//         let _ = sender.send(PathBuf::from("TESTM"));

//         let mut root_path = Crawler::root_from_channel(&receiver, 100);
//         assert_eq!(root_path.unwrap(), Some("TESTM".to_string()));

//         root_path = Crawler::root_from_channel(&receiver, 1);
//         assert_eq!(root_path.unwrap(), None)
//     }
// }
