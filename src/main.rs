mod indexer;
mod constants;
mod crawler;

use std::env;
// use std::error::Error;
// use std::result::Result;
// use anyhow::{Context, Result, Error, anyhow};
use anyhow::{Context, Result, anyhow};

use std::thread;
use crossbeam::channel::unbounded;
use crossbeam::channel::{Sender, Receiver};
use std::path::PathBuf;


fn main() -> Result<()> {
    let root = env::args().nth(1).unwrap_or_else(|| constants::DEFAULT_ROOT.to_string());
    let (crawler_chan, processor_chan): (Sender<PathBuf>, Receiver<PathBuf>)  = unbounded();

    let (file_chan, index_chan):(Sender<String>, Receiver<String>) = unbounded();
    let mut crawler_threads = vec![];

    let indexer_thread = thread::spawn(move || {
        indexer::build_index(index_chan).unwrap();
    });

    crawler_chan.send(PathBuf::from(root)).context("Failed to send root")?;

    for _ in 1..=constants::MAX_THREAD {
        let crawler = crawler_chan.clone();
        let processor = processor_chan.clone();
        let results = file_chan.clone();
        crawler_threads.push(thread::spawn(move || {
            crawler::crawl_this(crawler, processor, results);
        }));
    }


    for (id, c) in crawler_threads.into_iter().enumerate() {
            println!("Waiting on thread {}", id);
            c.join().expect("Runtime issue while waiting on crawler threads");
    }
    drop(file_chan);

    indexer_thread.join().expect("Runtime issue while waiting on indexer thread");

    if ! processor_chan.is_empty() {
        return Err(anyhow!("Failed to crawl everything since processor chan is still non empty"));
    }

    Ok(())
}
