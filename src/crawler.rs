use walkdir::WalkDir;
use crossbeam::channel::select;
use std::time::Duration;
use crossbeam::channel::{Sender, Receiver};
use std::path::PathBuf;
use std::thread;
use crate::constants::CHAN_TIMEOUT_S;

fn root_from_channel(receiver: &Receiver<PathBuf>) -> Option<String> {
    select! {
        recv(receiver) -> msg => {
            Some(msg.unwrap().to_str().unwrap().to_string())
        },
        default(Duration::from_secs(CHAN_TIMEOUT_S)) => None,
    }
}

pub fn crawl_this(sender: Sender<PathBuf>, receiver: Receiver<PathBuf>, result: Sender<String>) {

    let mut root:String;
    let whoami: String = format!("{:?}", thread::current().id());
    if receiver.is_empty() {
        println!("Empty channel for {}, exiting early!", whoami);
        return
    }

    // Blocks for the first time in 1+ threads.
    println!("Crawling in thread {}", whoami);


    loop {
        let some_root = root_from_channel(&receiver);
        if some_root.is_none() {
            println!("Crawling done in {}, leaving, bye!", whoami);
            return;
        } 
        root = some_root.unwrap();
        // println!("{} crawling {}", whoami, root);
        for entry in WalkDir::new(&root).max_depth(1).into_iter().skip(1) {
            match entry {
                Ok(dirent) => {
                    match dirent.metadata() {
                        Ok(metadata) => {
                            if metadata.is_dir() {
                                let dirpath = dirent.path().to_path_buf().to_owned();

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
                },
                Err(err) => println!("Ignoring entry due to {}", err),

            }
        }
    }
}