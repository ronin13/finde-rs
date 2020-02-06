/// Default root to start crawling from.
pub const DEFAULT_ROOT: &str = "/usr/lib";

/// Initial number of threads in the threadpool.
pub const INIT_THREADS: usize = 3;

/// Maximum threads in threadpool.

pub const MAX_THREADS: usize = 14;

/// The watermark beyond which scheduler starts
/// scaling up the threadpool upto
/// ``` MAX_THREADS ```.
pub const THROTTLE_WMARK: usize = 10;

/// Duration between threadpool evals
/// by scheduler.
pub const SCHED_SLEEP_S: u64 = 1;

/// Timeout in seconds for the amount of time
/// each thread in pool wait for any directory
/// paths to crawl
pub const CHAN_TIMEOUT_S: u64 = 10;

/// Default location of Index directory
/// for use by tantivy.
/// Must exist and be empty!
pub const INDEX_DIR: &str = "/tmp/index";
