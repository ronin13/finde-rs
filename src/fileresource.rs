use std::thread;

use crate::constants::CHAN_TIMEOUT_S;
use crate::resource::Resource;
use log::{debug, info, trace, warn};
use walkdir::WalkDir;
// use anyhow::{anyhow, Context, Result};
use anyhow::{anyhow, Result};
use crossbeam::channel::select;
use crossbeam::channel::{Receiver, Sender};
use std::path::PathBuf;
use std::time::Duration;
#[derive(Debug, Clone)]
pub struct FileResource {
    path: PathBuf,
}

impl FileResource {
    pub fn new(path: String) -> FileResource {
        FileResource {
            path: PathBuf::from(path),
        }
    }
    fn root_from_channel(receiver: &Receiver<PathBuf>, timeout: u64) -> Result<Option<String>> {
        select! {
            recv(receiver) -> msg => {
                let message = msg?;
                let res = FileResource::pathbuf_to_string(message)?;
                Ok(Some(res))

            },
            default(Duration::from_secs(timeout)) => Ok(None),
        }
    }
    fn pathbuf_to_string(_str: PathBuf) -> Result<String> {
        let path_to_str = _str.to_str();
        match path_to_str {
            Some(pstr) => Ok(pstr.to_string()),
            None => Err(anyhow!("Failed to decode to String")),
        }
    }
}

impl Resource<PathBuf> for FileResource {
    fn crawl_this(
        &self,
        sender: Sender<PathBuf>,
        receiver: Receiver<PathBuf>,
        result: Sender<String>,
    ) -> Result<()> {
        let mut root: String;
        let whoami: String = format!("{:?}", thread::current().id());
        debug!("Crawling in thread {}", whoami);

        loop {
            let _root = FileResource::root_from_channel(&receiver, CHAN_TIMEOUT_S)?;
            if _root.is_none() {
                info!("Crawling done in {}, leaving, bye!", whoami);
                return Ok(());
            }
            root = _root.unwrap();
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
                // info!("Sending {:?} to channel", fil);
                result.send(fil)?;
            }
        }
    }

    #[allow(clippy::identity_conversion)]
    fn to_resource(&self) -> Result<PathBuf> {
        Ok(PathBuf::from(self.path.clone()))
    }
}
