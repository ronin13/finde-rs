[![Build Status](https://travis-ci.org/ronin13/finde-rs.svg?branch=master)](https://travis-ci.org/ronin13/finde-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![dependency status](https://deps.rs/repo/github/ronin13/finde-rs/status.svg)](https://deps.rs/repo/github/ronin13/finde-rs)
[![Cargo](https://img.shields.io/crates/v/finde-rs.svg)](https://crates.io/crates/finde-rs)
[![](https://docs.rs/finde-rs/badge.svg)](https://docs.rs/finde-rs)


finde-rs
--------

This is a CLI tool written in Rust, which indexes the directory with a multi-threaded crawler.
It uses [crossbeam](https://github.com/crossbeam-rs/crossbeam) for channels, [threadpool](https://github.com/rust-threadpool/rust-threadpool)
library for threadpool support and finally, [tantivy](https://github.com/tantivy-search/tantivy) for full-text indexing support.

### It has three main components:
+ Filecrawler is responsible for spawning indexer and scheduler threads. It creates a threadpool whose threads are used for walking the directory tree. It creates the crossbeam channels to communicate with indexer and to communicate between threadpool threads.  In each of the 'walks', directories are sent to the channel used by other threadpool threads as source for further crawling, and fully qualified file paths are sent to the indexer thread.
+ Scheduler is responsible for scaling the Filecrawler's threadpool up or down depending on the length of the threadpool's channels. It maintains the the threadpool size between a min and a max bound.
+ Indexer is a single threaded tantivy indexer which reads fully qualified file paths from channel sent by the Filecrawler's threads. It commits the index to disk once all the crawling has been done.


##  Usage

```sh

>> git clone https://github.com/ronin13/finde-rs && cd finde-rs
>> cargo build --release

>>./target/release/finde-rs --help
finde-rs 0.1.3
CLI finder tool

USAGE:
    finde-rs [FLAGS] [OPTIONS]

FLAGS:
    -h, --help
            Prints help information

    -q, --quiet
            Pass many times for less log output

    -V, --version
            Prints version information

    -v, --verbose
            Pass many times for more log output

            By default, it'll only report errors. Passing `-v` one time also prints warnings, `-vv` enables info
            logging, `-vvv` debug, and `-vvvv` trace.

OPTIONS:
    -I, --index-dir <index-dir>
            Root path to crawl from [default: /tmp/]

    -i, --initial-threads <initial-threads>
            Initial number of threads to spawn

    -m, --max-threads <max-threads>
            Maximum number of threads that threadpool can scale upto. Defaults to number of cpus

    -p, --path <path>
            Root path to crawl from [default: /usr/lib]


```

## Running

```sh

>>./target/release/finde-rs -p $HOME/repo -v -i 6 -m 12 --index-dir /tmp
2020-02-15 14:10:59,683 INFO  [finde_rs] Crawling /Users/raghu/repo
2020-02-15 14:10:59,684 INFO  [finde_rs::indexer] Starting indexer
2020-02-15 14:10:59,684 INFO  [finde_rs::crawler] Waiting on upto 12 crawler threads
2020-02-15 14:10:59,684 INFO  [finde_rs::indexer] Index directory created in /tmp/5ryH1
2020-02-15 14:10:59,684 INFO  [tantivy::indexer::segment_updater] save metas
2020-02-15 14:10:59,687 INFO  [finde_rs::indexer] Iterating over results
2020-02-15 14:10:59,785 INFO  [finde_rs::scheduler] Updating number of threads to 7, length of work queue 3818, pool size 6
2020-02-15 14:10:59,886 INFO  [finde_rs::scheduler] Updating number of threads to 8, length of work queue 6883, pool size 6
2020-02-15 14:10:59,988 INFO  [finde_rs::scheduler] Updating number of threads to 9, length of work queue 11192, pool size 6
2020-02-15 14:11:00,089 INFO  [finde_rs::scheduler] Updating number of threads to 10, length of work queue 12956, pool size 6
2020-02-15 14:11:00,190 INFO  [finde_rs::scheduler] Updating number of threads to 11, length of work queue 12857, pool size 6
2020-02-15 14:11:00,290 INFO  [finde_rs::scheduler] Updating number of threads to 12, length of work queue 12607, pool size 6
2020-02-15 14:11:04,834 INFO  [finde_rs::scheduler] Updating number of threads to 6, length of work queue 0, pool size 6
2020-02-15 14:11:05,739 INFO  [finde_rs::fileresource] Crawling done in ThreadId(5), leaving, bye!
2020-02-15 14:11:05,740 INFO  [finde_rs::fileresource] Crawling done in ThreadId(4), leaving, bye!
2020-02-15 14:11:05,740 INFO  [finde_rs::fileresource] Crawling done in ThreadId(2), leaving, bye!
2020-02-15 14:11:05,740 INFO  [finde_rs::fileresource] Crawling done in ThreadId(7), leaving, bye!
2020-02-15 14:11:05,740 INFO  [finde_rs::fileresource] Crawling done in ThreadId(6), leaving, bye!
2020-02-15 14:11:05,740 INFO  [finde_rs::fileresource] Crawling done in ThreadId(3), leaving, bye!
2020-02-15 14:11:05,740 INFO  [finde_rs::indexer] Commiting the index
2020-02-15 14:11:05,740 INFO  [tantivy::indexer::index_writer] Preparing commit
2020-02-15 14:11:05,757 INFO  [finde_rs::scheduler] No more threads to schedule, I am done. Bye!
2020-02-15 14:11:05,899 INFO  [tantivy::indexer::segment_updater] Starting merge  - [Seg("8cc31b4d"), Seg("97576eb1"), Seg("2b7bcba3"), Seg("f1bbcb09"), Seg("4c3cf582"), Seg("699c0c3b"), Seg("4e08a0dd"), Seg("1e6b5009")]
2020-02-15 14:11:05,904 INFO  [tantivy::indexer::index_writer] Prepared commit 500530
2020-02-15 14:11:05,904 INFO  [tantivy::indexer::prepared_commit] committing 500530
2020-02-15 14:11:05,904 INFO  [tantivy::indexer::segment_updater] save metas
2020-02-15 14:11:05,905 INFO  [tantivy::indexer::segment_updater] Running garbage collection
2020-02-15 14:11:05,905 INFO  [tantivy::directory::managed_directory] Garbage collect
2020-02-15 14:11:05,905 INFO  [finde_rs::indexer] Index created in "/tmp/"
2020-02-15 14:11:05,905 INFO  [finde_rs::indexer] Index has 12 segments
2020-02-15 14:11:05,906 INFO  [finde_rs] Finished crawling /Users/raghu/repo, took 6s
./target/release/finde-rs -p $HOME/repo -v -i 6 -m 12 --index-dir   12.81s user 26.84s system 636% cpu 6.232 total


```


## Tests

```
>>cargo test
   Compiling finde-rs v0.1.1 (/Users/raghu/repo/finde-rs)
    Finished test [unoptimized] target(s) in 1.22s
     Running target/debug/deps/finde_rs-c62a74cfdff79a3e

running 3 tests
test scheduler::test::test_scale_with_bounds ... ok
test crawler::test::test_root_from_disconnected_channel ... ok
test crawler::test::test_root_from_channel ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out


```

## Linting

```
cargo clippy
```