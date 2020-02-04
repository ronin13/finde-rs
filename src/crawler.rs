use crate::constants::CHAN_TIMEOUT_S;
use anyhow::{anyhow, Result};
use crossbeam::channel::select;
use crossbeam::channel::{Receiver, Sender};
use log::{debug, info, trace, warn};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use walkdir::WalkDir;

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

pub fn crawl_this(
    sender: Sender<PathBuf>,
    receiver: Receiver<PathBuf>,
    result: Sender<String>,
) -> Result<()> {
    let mut root: String;
    let whoami: String = format!("{:?}", thread::current().id());
    debug!("Crawling in thread {}", whoami);

    loop {
        root = root_from_channel(&receiver)?;
        if root.is_empty() {
            info!("Crawling done in {}, leaving, bye!", whoami);
            return Ok(());
        }
        let mut filevec: Vec<String> = vec![];

        trace!("{} crawling {}", whoami, root);
        for entry in WalkDir::new(&root).max_depth(1).into_iter().skip(1) {
            match entry {
                Ok(dirent) => match dirent.metadata() {
                    Ok(metadata) => {
                        if metadata.is_dir() {
                            sender
                                .send(dirent.path().to_path_buf().to_owned())
                                .expect("Failed to send. Boo!");
                        } else {
                            filevec.push(dirent.path().to_str().unwrap().to_string());
                        }
                    }
                    Err(e) => warn!("Ignoring due to error {}", e),
                },
                Err(err) => warn!("Ignoring entry due to {}", err),
            }
        }
        for fil in filevec.into_iter() {
            result.send(fil)?;
        }
    }
}
