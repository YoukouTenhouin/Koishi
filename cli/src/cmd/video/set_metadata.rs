use clap::Parser;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::{
    error::Error,
    fs::File,
    path::{Path, PathBuf},
    result::Result,
};

use crate::api_req::{self, APIResponseWithData};

#[derive(Parser)]
pub(super) struct Args {
    uuid: String,
    path: PathBuf,
}

#[derive(Deserialize)]
struct ResUploadMetadata {
    url: String,
}

fn do_upload_metadata(uuid: &str, metadata: &Path) -> Result<(), Box<dyn Error>> {
    let url = format!("video/{uuid}/upload_metadata");
    let res: APIResponseWithData<ResUploadMetadata> = api_req::post(url)
        .send()?
	.json()?;
    let res = res.unwrap_or_error_out();

    let f = File::open(metadata)?;

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()?;
    client.put(res.url)
	.header("Content-Type", "application/xml")
	.body(f)
	.send()?;
    Ok(())
}

pub(crate) fn main(args: Args) {
    std::println!("Uploading metadata file {path}", path=args.path.display());

    let ret = do_upload_metadata(&args.uuid, &args.path);
    match ret {
	Ok(()) => println!("Upload finished"),
	Err(e) => {
	    eprintln!("Error occured during uploading: {e}");

            let mut source = e.source();
            while let Some(e) = source {
		eprintln!("Caused by: {}", e);
		source = e.source();
            }
	}
    }
}
