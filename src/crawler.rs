use walkdir::WalkDir;
use crossbeam::channel::{Sender, Receiver};
use std::path::PathBuf;

pub fn crawl_this(sender: Sender<PathBuf>, receiver: Receiver<PathBuf>, result: Sender<String>) {

    if receiver.is_empty() {
        return
    }

    // Blocks for the first time in 1+ threads.
    let mut root = receiver.recv().unwrap().to_str().unwrap().to_string();

    loop {
        println!("Crawling {}", root);
        for entry in WalkDir::new(&root).max_depth(1).into_iter().skip(1) {
            match entry {
                Ok(dirent) => {
                    match dirent.metadata() {
                        Ok(metadata) => {
                            if metadata.is_dir() {
                                let dirpath = dirent.path().to_path_buf().to_owned();

                                sender.send(dirpath).expect("Failed to send. Boo!");
                                println!("{} is a directory", dirent.path().display());
                            }  else { 
                                let fpath = dirent.path().to_str().unwrap().to_string();
                                println!("RESULT: {} is a file", fpath);
                                result.send(fpath).expect("Failed to send");
                            }

                        },
                        Err(e) => println!("Ignoring due to error {}", e),
                    }
                },
                Err(err) => println!("Ignoring entry due to {}", err),

            }
        }
        if receiver.is_empty() {
            return
        }
        root = receiver.recv().unwrap().to_str().unwrap().to_string();
    }
}