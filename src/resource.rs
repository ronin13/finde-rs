use anyhow::Result;
use std::str::FromStr;

use std::marker::{Send, Sync};

#[derive(Debug)]
pub enum Response<T> {
    DirFileResponse { dirs: Vec<T>, files: Vec<String> },
}

pub trait Resource<T: FromStr + Send + Sync>: Send + Sync {
    fn get_dirs_and_leaves(&self, path: &T) -> Response<T>;
    fn get_path(&self) -> Result<T>;
}

// trait alias not stable yet!
// trait StringyTVar = FromStr + Send + Sync + std::fmt::Debug;
