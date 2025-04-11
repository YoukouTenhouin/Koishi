use blake2::{Blake2b512, Digest};
use clap::Parser;
use hex;
use reqwest::blocking::Client;
use std::{
    fs::File, io::Read, path::{Path, PathBuf}
};

use crate::api;

#[derive(Parser)]
pub(super) struct Args {
    uuid: String,
    path: PathBuf,
}

fn hash(content: &[u8]) -> String {
    let digest = Blake2b512::digest(content);
    hex::encode(digest)
}

fn upload_cover(content: Vec<u8>) -> api::Result<String> {
    let hash = hash(content.as_slice());

    let res_body = api::cover::upload_url(hash.clone())?;

    if res_body.exists {
	println!("Cover already uploaded, skipping");
	Ok(hash)
    } else {
	let client = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()?;
	client.put(res_body.url.unwrap())
	    .header("Content-Type", "image/jpeg")
	    .body(content)
	    .send()?;
	println!("Cover uploaded");
	Ok(hash)
    }
}

fn do_upload(
    uuid: &str, path: &Path
) -> Result<(), Box<dyn std::error::Error>> {
    let mut f = File::open(path)?;
    let f_size = f.metadata()?.len();
    let mut buf = vec![0; f_size as usize];
    f.read(&mut buf)?;

    let cover = upload_cover(buf)?;
    api::video::update(uuid, None, Some(cover), None)?;

    Ok(())
}

pub(crate) fn main(args: Args) {
    do_upload(&args.uuid, &args.path).unwrap();
}
