use blake2::{Blake2b512, Digest};
use clap::Parser;
use hex;
use reqwest::blocking::Client;
use serde::{Serialize, Deserialize};
use std::{
    fs::File, io::Read, path::{Path, PathBuf}
};

use crate::api_req::{self, APIResponse, APIResponseWithData};

#[derive(Parser)]
pub(super) struct Args {
    uuid: String,
    path: PathBuf,
}

#[derive(Serialize)]
struct ReqCover {
    hash: String,
}

#[derive(Serialize)]
struct ReqVideoUpdate {
    cover: String,
}

#[derive(Deserialize)]
struct ResCover {
    exists: bool,
    url: Option<String>,
}

fn hash(content: &[u8]) -> String {
    let digest = Blake2b512::digest(content);
    hex::encode(digest)
}

fn upload_cover(content: Vec<u8>) -> api_req::Result<String> {
    let hash = hash(content.as_slice());

    let req_body = ReqCover {
	hash: hash.clone(),
    };
    let res = api_req::post("cover")
        .json(&req_body)
        .send()?;
    let res_body: APIResponseWithData<ResCover> = res.json()?;
    let res_body = res_body.unwrap_or_error_out();

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

fn update_video(uuid: &str, cover: &str) -> api_req::Result<()> {
    let url = format!("video/{uuid}");
    let req_body = ReqVideoUpdate{ cover: cover.to_string() };
    let res: APIResponse = api_req::put(url)
        .json(&req_body)
        .send()?
	.json()?;

    Ok(res.unwrap_or_error_out())
}

fn do_upload(
    uuid: &str, path: &Path
) -> Result<(), Box<dyn std::error::Error>> {
    let mut f = File::open(path)?;
    let f_size = f.metadata()?.len();
    let mut buf = vec![0; f_size as usize];
    f.read(&mut buf)?;

    let cover = upload_cover(buf)?;
    update_video(uuid, cover.as_str())?;

    Ok(())
}

pub(crate) fn main(args: Args) {
    let ret = do_upload(&args.uuid, &args.path);
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
