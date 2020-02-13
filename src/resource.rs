use anyhow::Result;
use crossbeam::channel::{Receiver, Sender};
use std::str::FromStr;
// use std::clone::Clone;

use std::marker::{Send, Sync};
pub trait Resource<T: FromStr + Send + Sync>: Send + Sync {
    fn crawl_this(
        &self,
        sender: Sender<T>,
        receiver: Receiver<T>,
        result: Sender<String>,
    ) -> Result<()>;
    fn to_resource(&self) -> Result<T>;
}
