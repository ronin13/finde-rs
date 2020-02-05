use crate::constants;
use crossbeam::channel::Receiver;
use log::{debug, info};
use std::cmp::max;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use threadpool::ThreadPool;

/// We implement a scheduler here which dynamically adjusts the threadpool
/// in accordance with size of the request queue.
/// Scheduler also ensures threads do not exceed the ```constants::THROTTLE_WMARK```.
/// It also sleeps for ``` constants::SCHED_SLEEP_S``` seconds before re-evaluating.
pub fn run(mut pool: ThreadPool, processor: Receiver<PathBuf>) -> thread::JoinHandle<()> {
    let scheduler_thread = thread::spawn(move || {
        let mut current_threads: usize = pool.active_count();
        loop {
            if current_threads == 0 {
                info!("No more threads to schedule, I am done. Bye! ");
                break;
            }
            let len_of_processor = processor.len();
            if len_of_processor > constants::THROTTLE_WMARK {
                current_threads = max(current_threads + 1, constants::MAX_THREADS);
                info!("Increasing threads to {}", current_threads);
                pool.set_num_threads(current_threads);
            } else {
                info!("Current threads {}", current_threads);
            }
            debug!("Sleeping before runtime eval");
            thread::sleep(Duration::from_secs(constants::SCHED_SLEEP_S));
            current_threads = pool.active_count();
        }
    });
    return scheduler_thread;
}
