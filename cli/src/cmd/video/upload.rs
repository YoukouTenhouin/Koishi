use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf}
};
use tabled::{Table, settings::Style};

use crate::api_req::{self, APIResponseWithData};

use super::VideoInfoWithoutID;

#[derive(Parser)]
pub(super) struct Args {
    #[arg(short, long)]
    progress: bool,
    uuid: String,
    path: PathBuf,
}

#[derive(Deserialize)]
struct ResPresignedUrl {
    url: String,
    video: VideoInfoWithoutID
}

fn get_presigned_url(uuid: &str) -> api_req::Result<ResPresignedUrl> {
    let res = api_req::post(format!("video/{uuid}/upload_url")).send()?;
    let body: APIResponseWithData<ResPresignedUrl> = res.json()?;

    Ok(body.unwrap_or_error_out())
}

fn print_video_info(i: &VideoInfoWithoutID) {
    let mut table = Table::new(&[i]);
    table.with(Style::modern());

    println!("{table}");
}

fn do_upload<P: AsRef<Path>, S: AsRef<str>>(
    path: P, url: S, progress: bool) -> Result<(), Box<dyn std::error::Error>> {
    let f = File::open(path.as_ref())?;
    let f_size = f.metadata()?.len();
    let buf_reader = BufReader::new(f);

    let body = if progress {
	let pb = ProgressBar::new(f_size);
	pb.set_style(
            ProgressStyle::default_bar()
		.template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
		.unwrap()
		.progress_chars("#>-"),
	);
	reqwest::blocking::Body::sized(pb.wrap_read(buf_reader), f_size)
    } else {
	reqwest::blocking::Body::sized(buf_reader, f_size)
    };
    let client = Client::builder()
        .timeout(None)
        .build()?;
    client
	.put(url.as_ref())
	.header("Content-Type", "video/mp4")
	.header("Content-Length", f_size)
	.body(body)
	.send()?;
    Ok(())
}

pub(crate) fn main(args: Args) {
    std::println!("Obtaining pre-signed S3 URL");
    let result = get_presigned_url(args.uuid.as_str());

    if let Err(e) = result {
	eprintln!("API Request Error: {e}");
	std::process::exit(-1)
    }
    let result = result.unwrap();

    print_video_info(&result.video);

    std::println!("Uploading video file {path}", path=args.path.display());
    let ret = do_upload(args.path, result.url, args.progress);
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
