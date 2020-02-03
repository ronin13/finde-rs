use walkdir::WalkDir;
use crossbeam::channel::select;
use std::time::Duration;
use crossbeam::channel::{Sender, Receiver};
use std::path::PathBuf;
use std::thread;
use crate::constants::CHAN_TIMEOUT_S;
use anyhow::{Result, anyhow};
use log::{info, debug, warn, trace};


fn pathbuf_to_string(pbuf: PathBuf) -> Result<String> {
    let path_to_str = pbuf.to_str();
    match path_to_str {
        Some(pstr) => Ok(pstr.to_string()),
        None => Err(anyhow!("Failed to decode to String")),
    }
}

fn root_from_channel(receiver: &Receiver<PathBuf>) -> Result<String> {
    select! {
        recv(receiver) -> msg => {
            let message = msg?;
            pathbuf_to_string(message)
        },
        default(Duration::from_secs(CHAN_TIMEOUT_S)) => Ok(String::new()),
    }
}

pub fn crawl_this(sender: Sender<PathBuf>, receiver: Receiver<PathBuf>, result: Sender<String>) -> Result<()> {

    let mut root:String;
    let whoami: String = format!("{:?}", thread::current().id());
    debug!("Crawling in thread {}", whoami);


    loop {
        root = root_from_channel(&receiver)?;
        if root.is_empty() {
            info!("Crawling done in {}, leaving, bye!", whoami);
            return Ok(());
        } 
        trace!("{} crawling {}", whoami, root);
        for entry in WalkDir::new(&root).max_depth(1).into_iter().skip(1) {
            match entry {
                Ok(dirent) => {
                    match dirent.metadata() {
                        Ok(metadata) => {
                            if metadata.is_dir() {
                                let dirpath = dirent.path().to_path_buf().to_owned();

                                sender.send(dirpath).expect("Failed to send. Boo!");
                                trace!("{} is a directory", dirent.path().display());
                            }  else { 
                                let fpath = dirent.path().to_str().unwrap().to_string();
                                trace!("RESULT: {} is a file", fpath);
                                result.send(fpath).expect("Failed to send");
                            }

                        },
                        Err(e) => warn!("Ignoring due to error {}", e),
                    }
                },
                Err(err) => warn!("Ignoring entry due to {}", err),

            }
        }
    }
}
