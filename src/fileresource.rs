use crate::resource::Resource;
use crate::resource::Response;
use anyhow::Result;
use log::warn;
use std::path::PathBuf;
use walkdir::WalkDir;
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
}

impl Resource<PathBuf> for FileResource {
    fn get_dirs_and_leaves(&self, path: &PathBuf) -> Response<PathBuf> {
        let mut dirs = Vec::<PathBuf>::new();
        let mut files = Vec::<String>::new();

        for entry in WalkDir::new(path).max_depth(1).into_iter().skip(1) {
            match entry {
                Ok(dirent) => match dirent.metadata() {
                    Ok(metadata) => {
                        if metadata.is_dir() {
                            dirs.push(dirent.path().to_path_buf().to_owned());
                        } else {
                            let z = match dirent.path().to_str() {
                                Some(val) => val.to_string(),
                                None => {
                                    warn!("Error during conversion of {:?}", dirent);
                                    continue;
                                }
                            };
                            files.push(z);
                        }
                    }
                    Err(e) => warn!("Ignoring due to error {}", e),
                },
                Err(err) => warn!("Ignoring entry due to {}", err),
            }
        }

        Response::DirFileResponse { dirs, files }
    }

    fn get_path(&self) -> Result<PathBuf> {
        Ok(self.path.clone())
    }
}
