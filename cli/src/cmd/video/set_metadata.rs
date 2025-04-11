use clap::Parser;
use reqwest::blocking::Client;
use std::{
    error::Error,
    fs::File,
    path::{Path, PathBuf},
    result::Result,
};

use crate::api;
use crate::global_options;

#[derive(Parser)]
pub(super) struct Args {
    uuid: String,
    path: PathBuf,
}

fn do_upload_metadata(uuid: &str, metadata: &Path) -> Result<(), Box<dyn Error>> {
    let url = api::video::metadata_upload_url(uuid)?;

    if global_options::DRY.get().unwrap().clone() {
	return Ok(())
    }

    let f = File::open(metadata)?;

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()?;
    client.put(url)
	.header("Content-Type", "application/xml")
	.body(f)
	.send()?;
    Ok(())
}

pub(crate) fn main(args: Args) {
    std::println!("Uploading metadata file {path}", path=args.path.display());

    do_upload_metadata(&args.uuid, &args.path).unwrap();
}
