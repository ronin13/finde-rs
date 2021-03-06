use crate::constants;
use crate::haslen::HasLen;
use anyhow::Result;
use log::{debug, info};
use partial_application::partial;
use std::thread;
use std::time::Duration;
use threadpool::ThreadPool;

enum Scale {
    UP,
    DOWN,
}

/// Additive Increase, Multiplicative Decrease (AIMD)
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
            if current / 2 < lower {
                lower
            } else {
                current / 2
            }
        }
    }
}

/// We implement a scheduler here which dynamically adjusts the threadpool
/// in accordance with size of the request queue.
/// Scheduler also ensures threads do not exceed the ```constants::THROTTLE_WMARK```.
/// It also sleeps for ``` constants::SCHED_SLEEP_S``` seconds before re-evaluating.
pub fn run(
    mut pool: ThreadPool,
    processor: impl HasLen,
    initial_threads: usize,
    max_threads: usize,
) -> thread::JoinHandle<Result<()>> {
    thread::spawn(move || -> Result<()> {
        let mut current_threads: usize = pool.active_count();
        let mut required_threads;
        let mut pool_size;
        let mut len_of_processor;

        // Curried version of scaler with bounds applied.
        let current_scaler = partial!(scale_with_bounds => max_threads, initial_threads, _, _);

        loop {
            pool_size = pool.active_count();
            if pool_size == 0 {
                info!("No more threads to schedule, I am done. Bye! ");
                break;
            }
            len_of_processor = processor.len();

            if len_of_processor > constants::THROTTLE_WMARK {
                required_threads = current_scaler(current_threads, Scale::UP);
            } else {
                required_threads = current_scaler(current_threads, Scale::DOWN);
            }

            if required_threads != current_threads {
                info!(
                    "Updating number of threads to {}, length of work queue {}, pool size {}",
                    required_threads, len_of_processor, pool_size
                );
                pool.set_num_threads(required_threads);
                current_threads = required_threads;
            }
            debug!("Sleeping before runtime eval");
            thread::sleep(Duration::from_millis(constants::SCHED_SLEEP_MS));
        }
        Ok(())
    })
}

#[cfg(test)]
mod test {

    use crate::scheduler;

    #[test]
    fn test_scale_with_bounds() {
        assert_eq!(
            scheduler::scale_with_bounds(10, 0, 5, scheduler::Scale::UP),
            6
        );
        assert_eq!(
            scheduler::scale_with_bounds(10, 0, 10, scheduler::Scale::UP),
            10
        );
        assert_eq!(
            scheduler::scale_with_bounds(10, 1, 1, scheduler::Scale::DOWN),
            1
        );
    }
}
