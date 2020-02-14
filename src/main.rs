mod constants;
mod crawler;
mod fileresource;
mod haslen;
mod indexer;
mod resource;
mod scheduler;

use anyhow::{anyhow, Result};
use log::info;
use log::Level;
use std::time::Instant;
use structopt::StructOpt;

use crawler::Crawler;
use fileresource::FileResource;

#[derive(Debug, StructOpt)]
#[structopt(name = "finde-rs", about = "CLI finder tool")]
struct Opt {
    #[structopt(flatten)]
    verbose: clap_verbosity_flag::Verbosity,

    #[structopt(short, long, default_value = constants::DEFAULT_ROOT)]
    path: String,
}

/// Entry point of the finde-rs.
fn main() -> Result<()> {
    let opt = Opt::from_args();
    simple_logger::init_with_level(Level::Info)?;

    let start = Instant::now();

    match opt.path.chars().next() {
        Some('/') => {
            info!("Crawling {}", opt.path);
            let crawler = Crawler::new(Box::new(FileResource::new(opt.path.clone())));
            crawler.run()?;
        }
        _ => {
            return Err(anyhow!(
                "Crawling not implemented *yet* for non filesystem paths"
            ))
        }
    }

    info!(
        "Finished crawling {}, took {}s",
        opt.path,
        start.elapsed().as_secs()
    );

    Ok(())
}
