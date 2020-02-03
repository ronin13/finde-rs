mod indexer;
mod constants;
mod crawler;

use anyhow::{Context, Result, anyhow};
use structopt::StructOpt;
use log::{info,debug};
use simple_logger;
use log::Level;
use std::time::Duration;

use std::thread;
use crossbeam::channel::unbounded;
use crossbeam::channel::{Sender, Receiver};
use std::path::PathBuf;
use threadpool::ThreadPool;


#[derive(Debug, StructOpt)]
#[structopt(name = "finde-rs", about = "CLI finder tool")]
struct Opt {
    #[structopt(flatten)]
    verbose: clap_verbosity_flag::Verbosity,


    #[structopt(short, long, default_value = constants::DEFAULT_ROOT)]
    path: String,

}


fn main() -> Result<()> {
    simple_logger::init_with_level(Level::Info).unwrap();
    let pool = ThreadPool::new(constants::INIT_THREADS);


    let opt = Opt::from_args();
    let root = opt.path;
    let (crawler_chan, processor_chan): (Sender<PathBuf>, Receiver<PathBuf>)  = unbounded();

    let (file_chan, index_chan):(Sender<String>, Receiver<String>) = unbounded();

    let indexer_thread = thread::spawn(move || {
        indexer::build_index(index_chan).unwrap();
    });

    // Initial seed.
    crawler_chan.send(PathBuf::from(root)).context("Failed to send root path")?;

    #[allow(unused_must_use)]
    for _ in 1..=constants::INIT_THREADS {
        let crawler = crawler_chan.clone();
        let processor = processor_chan.clone();
        let results = file_chan.clone();
        pool.execute(move || {
            crawler::crawl_this(crawler, processor, results);
        });
    }

    let mut pool_t = pool.clone();
    let processor_t = processor_chan.clone();
    let scheduler_thread = thread::spawn(move || {
        let mut current_threads:usize = pool_t.active_count();
        loop {
            if current_threads == 0 {
                info!("No more threads to schedule, I am done. Bye! ");
                break;
            }
            let len_of_processor = processor_t.len();
            if len_of_processor > constants::THROTTLE_WMARK {
                current_threads += 1;
                current_threads %= constants::MAX_THREADS;
                info!("Increasing threads to {}", current_threads);
                pool_t.set_num_threads(current_threads);
            } else {
                info!("Current threads {}", current_threads);
            }
            debug!("Sleeping before runtime eval");
            thread::sleep(Duration::from_secs(constants::SCHED_SLEEP_S));
            current_threads = pool_t.active_count();
        }
    });

    info!("Waiting on upto {} crawler threads", constants::MAX_THREADS);
    pool.join();
    drop(file_chan);
    scheduler_thread.join().expect("Runtime issue with scheduler thread join");

    indexer_thread.join().expect("Runtime issue while waiting on indexer thread");

    if ! processor_chan.is_empty() {
        return Err(anyhow!("Failed to crawl everything since processor chan is still non empty"));
    }

    Ok(())
}
