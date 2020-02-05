mod constants;
mod crawler;
mod indexer;
mod scheduler;

use anyhow::{anyhow, Context, Result};
use log::info;
use log::Level;
use simple_logger;
use structopt::StructOpt;

use crossbeam::channel::unbounded;
use crossbeam::channel::{Receiver, Sender};
use std::path::PathBuf;
use std::thread;
use threadpool::ThreadPool;

#[derive(Debug, StructOpt)]
#[structopt(name = "finde-rs", about = "CLI finder tool")]
struct Opt {
    #[structopt(flatten)]
    verbose: clap_verbosity_flag::Verbosity,

    #[structopt(short, long, default_value = constants::DEFAULT_ROOT)]
    path: String,
}

/// Entry point of the finde-rs.
/// Initializes channels, logging and threadpools.
fn main() -> Result<()> {
    let opt = Opt::from_args();
    simple_logger::init_with_level(Level::Info).unwrap();
    let pool = ThreadPool::new(constants::INIT_THREADS);

    let root = opt.path;
    let (crawler_chan, processor_chan): (Sender<PathBuf>, Receiver<PathBuf>) = unbounded();

    let (file_chan, index_chan): (Sender<String>, Receiver<String>) = unbounded();

    let indexer_thread = thread::spawn(move || {
        indexer::build_index(index_chan).unwrap();
    });

    // Initial seed.
    crawler_chan
        .send(PathBuf::from(root))
        .context("Failed to send root path")?;

    for _ in 1..=constants::INIT_THREADS {
        let crawler = crawler_chan.clone();
        let processor = processor_chan.clone();
        let results = file_chan.clone();
        pool.execute(move || {
            crawler::crawl_this(crawler, processor, results).expect("Thread failed");
        });
    }

    // let mut pool_t = pool.clone();
    let pool_t = pool.clone();
    let processor_t = processor_chan.clone();
    let scheduler_thread = scheduler::run(pool_t, processor_t);

    info!("Waiting on upto {} crawler threads", constants::MAX_THREADS);
    pool.join();
    drop(file_chan);
    scheduler_thread
        .join()
        .expect("Runtime issue with scheduler thread join");

    indexer_thread
        .join()
        .expect("Runtime issue while waiting on indexer thread");

    if !processor_chan.is_empty() {
        return Err(anyhow!(
            "Failed to crawl everything since processor chan is still non empty"
        ));
    }

    Ok(())
}
