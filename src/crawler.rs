use crate::constants::INIT_THREADS;
use crate::haslen::HasLen;
use crate::indexer;
use crate::resource::Resource;
use crate::scheduler;
use anyhow::{anyhow, Context, Result};
use crossbeam::channel::unbounded;
use crossbeam::channel::{Receiver, Sender};
use log::info;
use std::str::FromStr;
use std::sync::Arc;
use std::thread;
use threadpool::ThreadPool;

use std::marker::{Send, Sync};

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
    initial_threads: usize,
    max_threads: usize,
}

impl<T: FromStr + Send + Sync + 'static> Crawler<T> {
    pub fn new(
        _resource: Box<dyn Resource<T>>,
        _initial_threads: Option<usize>,
        _max_threads: Option<usize>,
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
        }
    }

    fn run_indexer() -> (thread::JoinHandle<Result<()>>, Sender<String>) {
        let (file_chan, index_chan): (Sender<String>, Receiver<String>) = unbounded();
        (
            thread::spawn(move || -> Result<()> {
                match indexer::build_index(index_chan) {
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

    pub fn run(&self) -> Result<()> {
        let path = self.resource.get_path()?;

        self.crawler_chan.send(path)?;

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
