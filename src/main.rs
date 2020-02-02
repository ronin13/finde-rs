mod index;
mod constants;
mod crawler;

use std::env;
// use std::time;
use std::error::Error;
use std::result::Result;

use std::thread;
use crossbeam::channel::unbounded;
use crossbeam::channel::{Sender, Receiver};
// use std::fs::FileType;
// use std::option::Option;
use std::path::PathBuf;

// use std::sync::{Arc, Mutex};


fn main() -> Result<(), Box<dyn Error>> {
    let root = env::args().nth(1).unwrap_or_else(|| constants::DEF_ROOT.to_string());
    let (crawler, processor): (Sender<PathBuf>, Receiver<PathBuf>)  = unbounded();

    let (sresults, rresults):(Sender<String>, Receiver<String>) = unbounded();
    let mut cthreads = vec![];


    // let crawler_arc = Arc::new(Mutex::new(crawler));
    let ithread = thread::spawn(move || {
        index::build_index(rresults).unwrap();
    });


    crawler.send(PathBuf::from(root)).expect("Failed to send root");


    for _ in 1..=constants::MAX_THREAD {
        let icrawler = crawler.clone();
        let iprocessor = processor.clone();
        let isresults = sresults.clone();
        cthreads.push(thread::spawn(move || {
            crawler::crawl_this(icrawler, iprocessor, isresults);
        }));
    }


    for (id, c) in cthreads.into_iter().enumerate() {
            println!("Waiting on thread {}", id);
            c.join().expect("Runtime issue while waiting on crawler threads");
    }
    drop(sresults);

    ithread.join().expect("Runtime issue while waiting on indexer thread");
    // println!("Hello, world! {}", root);

    Ok(())
}
