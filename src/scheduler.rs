use crate::constants;
use crate::haslen::HasLen;
use log::{debug, info};
use partial_application::partial;
use std::thread;
use std::time::Duration;
use threadpool::ThreadPool;

enum Scale {
    UP,
    DOWN,
}

fn scale_with_bounds(upper: usize, lower: usize, current: usize, direction: Scale) -> usize {
    match direction {
        Scale::UP => {
            if current + 1 > upper {
                upper
            } else {
                current + 1
            }
        }
        Scale::DOWN => {
            if current - 1 < lower {
                lower
            } else {
                current - 1
            }
        }
    }
}

/// We implement a scheduler here which dynamically adjusts the threadpool
/// in accordance with size of the request queue.
/// Scheduler also ensures threads do not exceed the ```constants::THROTTLE_WMARK```.
/// It also sleeps for ``` constants::SCHED_SLEEP_S``` seconds before re-evaluating.
pub fn run(mut pool: ThreadPool, processor: impl HasLen) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut current_threads: usize = pool.active_count();
        let current_scaler =
            partial!(scale_with_bounds => constants::MAX_THREADS, constants::INIT_THREADS, _, _);
        loop {
            if pool.active_count() == 0 {
                info!("No more threads to schedule, I am done. Bye! ");
                break;
            }
            let len_of_processor = processor.len();
            if len_of_processor > constants::THROTTLE_WMARK {
                current_threads = current_scaler(current_threads, Scale::UP);
                info!(
                    "Increasing threads to {}, length of work queue {}",
                    current_threads, len_of_processor
                );
                pool.set_num_threads(current_threads);
            } else {
                current_threads = current_scaler(current_threads, Scale::DOWN);
                info!(
                    "Decreasing threads to {}, length of work queue {}",
                    current_threads, len_of_processor
                );
            }
            debug!("Sleeping before runtime eval");
            thread::sleep(Duration::from_secs(constants::SCHED_SLEEP_S));
        }
    })
}
