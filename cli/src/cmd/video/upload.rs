use clap::Parser;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use serde::{Serialize, Deserialize};
use std::{
    cmp::min, fs::File, io::{self, BufReader, Read, Seek, SeekFrom}, path::{Path, PathBuf}
};
use tabled::{Table, settings::Style};

use crate::api_req::{self, APIResponse, APIResponseWithData};

use super::VideoInfoWithoutID;

#[derive(Parser)]
pub(super) struct Args {
    #[arg(short, long)]
    progress: bool,
    #[arg(short='s', long, default_value_t=100_000_000)]
    part_size: u64,

    uuid: String,
    path: PathBuf,
}

#[derive(Serialize)]
struct ReqUploadStart {
    size: u64,
    part_size: u64
}

#[derive(Serialize)]
struct ReqUploadFinish {
    upload_id: String,
    etags: Vec<String>,
}

#[derive(Deserialize)]
struct ResUploadStart {
    urls: Vec<String>,
    upload_id: String,
    video: VideoInfoWithoutID
}

fn print_video_info(i: &VideoInfoWithoutID) {
    let mut table = Table::new(&[i]);
    table.with(Style::modern());

    println!("{table}");
}

struct UploadProgress {
    #[allow(dead_code)]
    mp: MultiProgress,
    pb_parts: ProgressBar,
    pb_total: ProgressBar,
    pb_current: ProgressBar,
}

struct UploadProgressReadWrapper<R: Read> {
    inner: R,
    pb_current: ProgressBar,
    pb_total: ProgressBar,
}

impl UploadProgress {
    fn new(parts: u64, total: u64) -> Self {
	let mp = MultiProgress::new();

	let pb_parts = mp.add(ProgressBar::new(parts));
	pb_parts.set_style(ProgressStyle::default_bar()
			   .template("Parts   {bar:40.cyan/blue} {pos}/{len}")
			   .unwrap()
			   .progress_chars("##-"));
	pb_parts.set_position(0);

	let pb_total = mp.add(ProgressBar::new(total));
	pb_total.set_style(ProgressStyle::default_bar()
			   .template("Total   {bar:40.green/black} {bytes}/{total_bytes}")
			   .unwrap());

	let pb_current = mp.add(ProgressBar::no_length());
	pb_current.set_style(ProgressStyle::default_bar()
			     .template("Current {bar:40.yellow/black} {bytes}/{total_bytes}")
			     .unwrap());

	Self{ mp, pb_parts, pb_total, pb_current }
    }

    fn start_part<R: Read>(&self, read: R, part_size: u64) -> UploadProgressReadWrapper<R> {
	self.pb_current.reset();
	self.pb_current.set_length(part_size);

	UploadProgressReadWrapper::<R>{
	    inner: read,
	    pb_total: self.pb_total.clone(),
	    pb_current: self.pb_current.clone(),
	}
    }

    fn finish_part(&self) {
	self.pb_parts.inc(1);
    }

    fn finish(&self) {
	self.pb_parts.finish();
	self.pb_total.finish();
	self.pb_current.finish();
    }
}

impl<R: Read> Read for UploadProgressReadWrapper<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
	let bytes_read = self.inner.read(buf)?;
	self.pb_current.inc(bytes_read as u64);
	self.pb_total.inc(bytes_read as u64);
	Ok(bytes_read)
    }
}

fn start_upload(uuid: &str, file_size: u64, part_size: u64) -> api_req::Result<ResUploadStart> {
    let url = format!("video/{uuid}/upload_start");
    let req_body = ReqUploadStart {
	size: file_size,
	part_size,
    };
    let res = api_req::post(url)
        .json(&req_body)
        .send()?;
    let res_body: APIResponseWithData<ResUploadStart> = res.json()?;
    Ok(res_body.unwrap_or_error_out())
}

fn upload_part(
    path: &Path, url: &str, offset: u64, size: u64, pb: &Option<UploadProgress>
) -> Result<String, Box<dyn std::error::Error>> {
    let mut f = File::open(path)?;
    f.seek(SeekFrom::Start(offset))?;

    let part_reader = f.take(size);
    let buf_reader = BufReader::new(part_reader);

    let body = if let Some(pb) = pb {
	reqwest::blocking::Body::sized(pb.start_part(buf_reader, size), size)
    } else {
	reqwest::blocking::Body::sized(buf_reader, size)
    };

    let client = Client::builder()
        .timeout(None)
        .build()?;
    let res = client.put(url)
	.header("Content-Type", "video/mp4")
	.body(body)
	.send()?;
    let etag = res.headers()
        .get("ETag")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    if let Some(pb) = pb {
	pb.finish_part();
    }

    Ok(etag)
}

fn finish_upload(uuid: &str, upload_id: String, etags: Vec<String>) -> api_req::Result<()> {
    let url = format!("video/{uuid}/upload_finish");
    let req_body = ReqUploadFinish{ upload_id, etags };

    let res = api_req::post(url)
        .json(&req_body)
        .send()?;
    let res: APIResponse = res.json()?;
    Ok(res.unwrap_or_error_out())
}

fn do_upload(
    uuid: &str, path: &Path, part_size: u64, progress: bool
) -> Result<(), Box<dyn std::error::Error>> {
    let f = File::open(path)?;
    let f_size = f.metadata()?.len();


    std::println!("Initiating multi-part upload");
    let upload_start = start_upload(uuid, f_size, part_size)?;
    print_video_info(&upload_start.video);

    let parts = upload_start.urls.len() as u64;

    let pb: Option<UploadProgress> = if progress {
	let pb = UploadProgress::new(parts, f_size);
	Some(pb)
    } else {
	None
    };

    let mut etags: Vec<String> = Vec::with_capacity(parts as usize);

    for (i, url) in upload_start.urls.iter().enumerate() {
	let offset = (i as u64) * part_size;
	let size = min(part_size, f_size - offset);
	let etag = upload_part(path, url, offset, size, &pb)?;
	etags.push(etag);
    }

    finish_upload(uuid, upload_start.upload_id, etags)?;
    pb.map(|v| v.finish());

    Ok(())
}

pub(crate) fn main(args: Args) {
    std::println!("Uploading video file {path}", path=args.path.display());

    let ret = do_upload(&args.uuid, &args.path, args.part_size, args.progress);
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
