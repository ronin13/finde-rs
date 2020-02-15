#![forbid(unsafe_code)]

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

    /// Root path to crawl from
    #[structopt(short, long, default_value = constants::DEFAULT_ROOT)]
    path: String,

    /// Maximum number of threads that threadpool can scale upto.
    /// Defaults to number of cpus.
    #[structopt(short, long)]
    max_threads: Option<usize>,

    /// Initial number of threads to spawn.
    #[structopt(short, long)]
    initial_threads: Option<usize>,
}

/// Entry point of the finde-rs.
fn main() -> Result<()> {
    let opt = Opt::from_args();
    let crawler;
    simple_logger::init_with_level(Level::Info)?;

    let start = Instant::now();

    match opt.path.chars().next() {
        Some('/') => {
            info!("Crawling {}", opt.path);
            crawler = Crawler::new(
                Box::new(FileResource::new(opt.path.clone())),
                opt.initial_threads,
                opt.max_threads,
            );
        }
        _ => {
            return Err(anyhow!(
                "Crawling not implemented *yet* for non filesystem paths"
            ))
        }
    }

    crawler.run()?;
    info!(
        "Finished crawling {}, took {}s",
        opt.path,
        start.elapsed().as_secs()
    );

    Ok(())
}
