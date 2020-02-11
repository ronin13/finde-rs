mod constants;
mod crawler;
mod haslen;
mod indexer;
mod scheduler;

use anyhow::Result;
use log::info;
use log::Level;
use structopt::StructOpt;

use crawler::FileCrawler;

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

    let crawler = FileCrawler::new(opt.path);
    crawler.run()?;
    info!("Finished crawling");

    Ok(())
}
