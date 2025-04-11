use clap::Parser;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use std::{
    cmp::min, fs::File, io::{self, BufReader, Read, Seek, SeekFrom}, path::{Path, PathBuf}
};
use tabled::{settings::{location::ByColumnName, Remove, Style}, Table};

use crate::api;

#[derive(Parser)]
pub(super) struct Args {
    #[arg(short, long)]
    progress: bool,
    #[arg(short='s', long, default_value_t=100_000_000)]
    part_size: u64,
    #[arg(short, long, default_value_t=10)]
    retry_part: u64,

    uuid: String,
    path: PathBuf,
}

fn print_video_info(i: &api::video::Video) {
    let mut table = Table::new(&[i]);
    table.with(Style::modern());
    table.with(Remove::column(ByColumnName::new("Cover URL")));

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

    fn rollback_part(&self)  {
	let rollback_length = self.pb_current.position();
	self.pb_current.dec(rollback_length);
	self.pb_total.dec(rollback_length);
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

fn do_upload_part(
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

fn upload_part(
    path: &Path, url: &str, offset: u64, size: u64, pb: &Option<UploadProgress>,
    retry: u64
) -> Result<String, Box<dyn std::error::Error>> {
    let mut retry_count = 0;
    loop {
	let result = do_upload_part(path, url, offset, size, pb);
	if result.is_ok() {
	    return result
	}

	retry_count += 1;
	if retry_count >= retry {
	    return result
	}
	pb.as_ref().map(|v| v.rollback_part());
    }
}


fn do_upload(
    uuid: &str, path: &Path, part_size: u64, progress: bool, retry: u64
) -> Result<(), Box<dyn std::error::Error>> {
    let f = File::open(path)?;
    let f_size = f.metadata()?.len();


    std::println!("Initiating multi-part upload");
    let upload_start = api::video::upload_start(uuid, f_size, part_size)?;
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
	let etag = upload_part(path, url, offset, size, &pb, retry)?;
	if !progress {
	    println!("Part uploaded {}/{}", i+1, parts);
	}
	etags.push(etag);
    }

    api::video::upload_finish(uuid, upload_start.upload_id, etags)?;
    pb.map(|v| v.finish());

    Ok(())
}

pub(crate) fn main(args: Args) {
    std::println!("Uploading video file {path}", path=args.path.display());

    do_upload(
	&args.uuid, &args.path, args.part_size, args.progress, args.retry_part)
    .unwrap();
   println!("Upload finished")
}
