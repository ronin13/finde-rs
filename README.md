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


## Running

```sh
>> git clone https://github.com/ronin13/finde-rs && cd finde-rs
>> cargo build --release
>>rm -rf /tmp/index/ && mkdir /tmp/index/ && ./target/release/finde-rs -p $HOME/repo
2020-02-11 22:09:37,989 INFO  [finde_rs::crawler] Waiting on upto 14 crawler threads
2020-02-11 22:09:37,989 INFO  [finde_rs::indexer] Starting indexer
2020-02-11 22:09:37,989 INFO  [tantivy::indexer::segment_updater] save metas
2020-02-11 22:09:38,045 INFO  [finde_rs::indexer] Iterating over results
2020-02-11 22:09:38,091 INFO  [finde_rs::scheduler] Updating number of threads to 4, length of work queue 445, pool size 3
2020-02-11 22:09:38,194 INFO  [finde_rs::scheduler] Updating number of threads to 5, length of work queue 719, pool size 3
2020-02-11 22:09:38,296 INFO  [finde_rs::scheduler] Updating number of threads to 6, length of work queue 948, pool size 3
2020-02-11 22:09:38,397 INFO  [finde_rs::scheduler] Updating number of threads to 7, length of work queue 1100, pool size 3
2020-02-11 22:09:38,501 INFO  [finde_rs::scheduler] Updating number of threads to 8, length of work queue 1560, pool size 3
2020-02-11 22:09:38,604 INFO  [finde_rs::scheduler] Updating number of threads to 9, length of work queue 1915, pool size 3
2020-02-11 22:09:38,709 INFO  [finde_rs::scheduler] Updating number of threads to 10, length of work queue 2456, pool size 3
2020-02-11 22:09:38,812 INFO  [finde_rs::scheduler] Updating number of threads to 11, length of work queue 2757, pool size 3
2020-02-11 22:09:38,912 INFO  [finde_rs::scheduler] Updating number of threads to 12, length of work queue 3106, pool size 3
2020-02-11 22:09:39,014 INFO  [finde_rs::scheduler] Updating number of threads to 13, length of work queue 3495, pool size 3
2020-02-11 22:09:39,114 INFO  [finde_rs::scheduler] Updating number of threads to 14, length of work queue 3587, pool size 3
2020-02-11 22:09:44,186 INFO  [tantivy::indexer::index_writer] Buffer limit reached, flushing segment with maxdoc=36069.
2020-02-11 22:09:45,308 INFO  [tantivy::indexer::index_writer] Buffer limit reached, flushing segment with maxdoc=34695.
2020-02-11 22:09:46,192 INFO  [tantivy::indexer::index_writer] Buffer limit reached, flushing segment with maxdoc=31888.
2020-02-11 22:09:47,247 INFO  [tantivy::indexer::index_writer] Buffer limit reached, flushing segment with maxdoc=30092.
2020-02-11 22:09:48,611 INFO  [tantivy::indexer::index_writer] Buffer limit reached, flushing segment with maxdoc=29200.
2020-02-11 22:09:48,698 INFO  [tantivy::indexer::index_writer] Buffer limit reached, flushing segment with maxdoc=28548.
2020-02-11 22:09:48,923 INFO  [tantivy::indexer::index_writer] Buffer limit reached, flushing segment with maxdoc=28221.
2020-02-11 22:09:48,949 INFO  [tantivy::indexer::index_writer] Buffer limit reached, flushing segment with maxdoc=28807.
2020-02-11 22:09:49,165 INFO  [tantivy::indexer::index_writer] Buffer limit reached, flushing segment with maxdoc=28801.
2020-02-11 22:09:49,166 INFO  [tantivy::indexer::index_writer] Buffer limit reached, flushing segment with maxdoc=28897.
2020-02-11 22:09:49,233 INFO  [tantivy::indexer::index_writer] Buffer limit reached, flushing segment with maxdoc=29073.
2020-02-11 22:09:49,308 INFO  [tantivy::indexer::index_writer] Buffer limit reached, flushing segment with maxdoc=28772.
2020-02-11 22:09:50,366 INFO  [tantivy::indexer::index_writer] Buffer limit reached, flushing segment with maxdoc=23442.
2020-02-11 22:09:50,692 INFO  [tantivy::indexer::index_writer] Buffer limit reached, flushing segment with maxdoc=22961.
2020-02-11 22:09:50,819 INFO  [tantivy::indexer::segment_updater] Starting merge  - [Seg("1796d91d"), Seg("c900e383"), Seg("ec70b10f"), Seg("16aa60ed"), Seg("39cbdc8e"), Seg("3d0ed2cf"), Seg("8008eb11"), Seg("6f2065bb")]
2020-02-11 22:09:50,875 INFO  [tantivy::indexer::index_writer] Buffer limit reached, flushing segment with maxdoc=21739.
2020-02-11 22:09:51,021 INFO  [tantivy::indexer::index_writer] Buffer limit reached, flushing segment with maxdoc=20742.
2020-02-11 22:09:52,164 INFO  [finde_rs::scheduler] Updating number of threads to 13, length of work queue 0, pool size 3
2020-02-11 22:09:52,266 INFO  [finde_rs::scheduler] Updating number of threads to 12, length of work queue 0, pool size 3
2020-02-11 22:09:52,369 INFO  [finde_rs::scheduler] Updating number of threads to 11, length of work queue 0, pool size 3
2020-02-11 22:09:52,470 INFO  [finde_rs::scheduler] Updating number of threads to 10, length of work queue 0, pool size 3
2020-02-11 22:09:52,570 INFO  [finde_rs::scheduler] Updating number of threads to 9, length of work queue 0, pool size 3
2020-02-11 22:09:52,671 INFO  [finde_rs::scheduler] Updating number of threads to 8, length of work queue 0, pool size 3
2020-02-11 22:09:52,763 INFO  [tantivy::indexer::segment_updater] Starting merge  - [Seg("f148a79a"), Seg("4ea4e78e"), Seg("300bb83b"), Seg("ba7ab83d"), Seg("842d1485"), Seg("fdcb2443"), Seg("1761fca7"), Seg("13adde49")]
2020-02-11 22:09:52,774 INFO  [finde_rs::scheduler] Updating number of threads to 7, length of work queue 0, pool size 3
2020-02-11 22:09:52,781 INFO  [tantivy::indexer::index_writer] Buffer limit reached, flushing segment with maxdoc=19071.
2020-02-11 22:09:52,874 INFO  [finde_rs::scheduler] Updating number of threads to 6, length of work queue 0, pool size 3
2020-02-11 22:09:52,977 INFO  [finde_rs::scheduler] Updating number of threads to 5, length of work queue 0, pool size 3
2020-02-11 22:09:53,079 INFO  [finde_rs::scheduler] Updating number of threads to 4, length of work queue 0, pool size 3
2020-02-11 22:09:53,181 INFO  [finde_rs::scheduler] Updating number of threads to 3, length of work queue 0, pool size 3
2020-02-11 22:09:53,344 INFO  [tantivy::indexer::index_writer] Buffer limit reached, flushing segment with maxdoc=17913.
2020-02-11 22:09:56,049 INFO  [tantivy::indexer::segment_updater] End merge Tracked(Some(InnerSegmentMeta { segment_id: Seg("40a23565"), max_doc: 247520, deletes: None }))
2020-02-11 22:09:56,049 INFO  [tantivy::indexer::segment_updater] Running garbage collection
2020-02-11 22:09:56,049 INFO  [tantivy::directory::managed_directory] Garbage collect
2020-02-11 22:09:56,060 INFO  [tantivy::directory::managed_directory] Deleted "c900e3831c154e7fa7eda0f88bf30a61.term"
2020-02-11 22:09:56,065 INFO  [tantivy::directory::managed_directory] Deleted "8008eb11dd764db19571ad9772a0bb17.term"
2020-02-11 22:09:56,071 INFO  [tantivy::directory::managed_directory] Deleted "39cbdc8e211641119539083853a6f5c9.store"
2020-02-11 22:09:56,077 INFO  [tantivy::directory::managed_directory] Deleted "8008eb11dd764db19571ad9772a0bb17.fieldnorm"
.....
.......
......
020-02-11 22:09:56,381 INFO  [tantivy::directory::managed_directory] Deleted "1796d91d0c3045b1ad40cf8d1902b7d1.posidx"
2020-02-11 22:09:56,387 INFO  [tantivy::directory::managed_directory] Deleted "16aa60ed09904d7c89476cb5a3d1c0f4.posidx"
2020-02-11 22:09:56,399 INFO  [tantivy::indexer::segment_updater] End merge Tracked(Some(InnerSegmentMeta { segment_id: Seg("0809d548"), max_doc: 204427, deletes: None }))
2020-02-11 22:09:56,399 INFO  [tantivy::indexer::segment_updater] Running garbage collection
2020-02-11 22:09:56,399 INFO  [tantivy::directory::managed_directory] Garbage collect
2020-02-11 22:09:56,405 INFO  [tantivy::directory::managed_directory] Deleted "f148a79a02dd4c9b9ff9215c9a2dabe7.posidx"
2020-02-11 22:09:56,410 INFO  [tantivy::directory::managed_directory] Deleted "13adde499f9f49fbb4e1118ad7dd1b86.pos"
...
...
2020-02-11 22:09:56,721 INFO  [tantivy::directory::managed_directory] Deleted "ba7ab83d5a9f4d84984a9140654a632e.idx"
2020-02-11 22:09:56,727 INFO  [tantivy::directory::managed_directory] Deleted "f148a79a02dd4c9b9ff9215c9a2dabe7.fieldnorm"
2020-02-11 22:09:56,733 INFO  [tantivy::directory::managed_directory] Deleted "ba7ab83d5a9f4d84984a9140654a632e.term"
2020-02-11 22:10:02,134 INFO  [finde_rs::crawler] Crawling done in ThreadId(4), leaving, bye!
2020-02-11 22:10:02,134 INFO  [finde_rs::crawler] Crawling done in ThreadId(2), leaving, bye!
2020-02-11 22:10:02,134 INFO  [finde_rs::crawler] Crawling done in ThreadId(3), leaving, bye!
2020-02-11 22:10:02,134 INFO  [finde_rs::indexer] Commiting the index
2020-02-11 22:10:02,134 INFO  [tantivy::indexer::index_writer] Preparing commit
2020-02-11 22:10:02,136 INFO  [finde_rs::scheduler] No more threads to schedule, I am done. Bye!
2020-02-11 22:10:03,179 INFO  [tantivy::indexer::segment_updater] Starting merge  - [Seg("c2c9ef75"), Seg("6819f3af"), Seg("65315b70"), Seg("7dcfec38"), Seg("8c8c6ad3"), Seg("6b5f306a"), Seg("b5c31021"), Seg("3e3ec5c0")]
2020-02-11 22:10:03,368 INFO  [tantivy::indexer::index_writer] Prepared commit 499568
2020-02-11 22:10:03,368 INFO  [tantivy::indexer::prepared_commit] committing 499568
2020-02-11 22:10:03,368 INFO  [tantivy::indexer::segment_updater] save metas
2020-02-11 22:10:03,380 INFO  [tantivy::indexer::segment_updater] Running garbage collection
2020-02-11 22:10:03,380 INFO  [tantivy::directory::managed_directory] Garbage collect
2020-02-11 22:10:03,386 INFO  [finde_rs::indexer] Index created in "/tmp/index"
2020-02-11 22:10:03,386 INFO  [finde_rs::indexer] Index has 16 segments
2020-02-11 22:10:03,387 INFO  [finde_rs] Finished crawling
./target/release/finde-rs -p $HOME/repo  12.79s user 25.45s system 148% cpu 25.719 total
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