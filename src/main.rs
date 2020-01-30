// #[macro_use]
// extern crate tantivy;

use std::env;
use std::error::Error;
use std::result::Result;
use walkdir::WalkDir;

// use tantivy::collector::TopDocs;
// use tantivy::query::QueryParser;
use tantivy::schema::*;
// use tantivy::{doc,Index, ReloadPolicy};
use tantivy::{doc,Index};
// use tantivy::error;


use std::thread;
use crossbeam::channel::unbounded;
use crossbeam::channel::{Sender, Receiver};
// use std::fs::FileType;
// use std::option::Option;
use std::path::PathBuf;

// use std::sync::{Arc, Mutex};
const DEF_ROOT: &str = "/tmp/yy";
// type OptStr = Option<&'static str>;


fn crawl_this(sender: Sender<PathBuf>, receiver: Receiver<PathBuf>, result: Sender<String>) {

    if receiver.is_empty() {
        return
    }

    let root = receiver.recv().unwrap().to_str().unwrap().to_string();

    for entry in WalkDir::new(&root) {
        match entry {
            Ok(dirent) => {
                match dirent.metadata() {
                    Ok(metadata) => {
                        if metadata.is_dir() {
                            let dirbuf = dirent.path().to_path_buf();

                            let dirpath = dirbuf.to_owned();

                            // let crawler_arc_clone = crawler_arc.clone();
                            // let sender = crawler_arc_clone.lock().expect("Failed");
                            sender.send(dirpath).expect("Failed to send. Boo!");
                            // println!("{} is a directory", dirent.path().display());
                        }  else { 
                            let fpath = dirent.path().to_str().unwrap().to_string();
                            // println!("RESULT: {} is a file", fpath);
                            result.send(fpath).expect("Failed to send");
                        }

                    },
                    Err(e) => println!("Ignoring due to error {}", e),
                }
                // println!("Entry {}", dirent.path().display());
            },
            Err(err) => println!("Ignoring entry due to {}", err),

        }
        // println!("{}", entry.path().display());
    }
}

fn build_index(results: Receiver<String>) -> Result<(), tantivy::TantivyError> {

    println!("Starting indexer");
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("rpath", TEXT | STORED);

    let schema = schema_builder.build();

    let index = Index::create_in_dir("/tmp/index", schema.clone())?;

    let mut index_writer = index.writer(50_000_000)?;


    let fpath = schema.get_field("rpath").unwrap();

    // let mut fpath_doc = Document::default();

    for fl in results.iter() {
        println!("Adding {}", fl);
        index_writer.add_document(doc!(
            fpath =>  fl,
        ));

    }

    index_writer.commit()?;

    Ok(())


}

fn main() -> Result<(), Box<dyn Error>> {
    let root = env::args().nth(1).unwrap_or_else(|| DEF_ROOT.to_string());
    let (crawler, processor): (Sender<PathBuf>, Receiver<PathBuf>)  = unbounded();

    let (sresults, rresults):(Sender<String>, Receiver<String>) = unbounded();
    let mut cthreads = vec![];


    // let crawler_arc = Arc::new(Mutex::new(crawler));

    crawler.send(PathBuf::from(root)).expect("Failed to send root");
    loop {
        if processor.is_empty() {
            println!("Done crawling, exiting");
            break;
        }
        let crawler = crawler.clone();
        let processor = processor.clone();
        let sresults = sresults.clone();
        cthreads.push(thread::spawn(move || {
            crawl_this(crawler, processor, sresults);
        }));

    }

    let ithread = thread::spawn(move || {
        build_index(rresults).unwrap();
    });

    for (id, c) in cthreads.into_iter().enumerate() {
            println!("Waiting on {}", id);
            c.join();
    }
    drop(sresults);

    ithread.join();
    // println!("Hello, world! {}", root);

    Ok(())
}
