use clap::Parser;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use rayon::prelude::*;
use std::{
    cmp::min,
    fs::File,
    io::{self, BufReader, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};
use tabled::{
    Table,
    settings::{Remove, Style, location::ByColumnName},
};

use crate::helpers::s3;
use crate::{api, helpers::cryptography::restricted_hash};

#[derive(Parser)]
pub(super) struct Args {
    #[arg(short = 'P', long)]
    no_progress: bool,
    #[arg(short = 's', long, default_value_t = 10_000_000)]
    part_size: u64,
    #[arg(short, long, default_value_t = 10)]
    retry_part: u64,
    #[arg(short, long)]
    password: Option<String>,
    #[arg(short, long)]
    thread_count: Option<usize>,

    uuid: String,
    path: PathBuf,
}

fn print_video_info(i: &api::video::Video) {
    let mut table = Table::new(&[i]);
    table.with(Style::modern());
    table.with(Remove::column(ByColumnName::new("Cover URL")));

    println!("{table}");
}

struct UploadMultiProgress {
    mp: MultiProgress,
    pb_parts: ProgressBar,
    pb_total: ProgressBar,
}

struct UploadProgress {
    mp: MultiProgress,
    pb_parts: ProgressBar,
    pb_total: ProgressBar,
    pb_current: ProgressBar,
}

struct UploadProgressReadWrapper<R: Read> {
    read: R,
    pb_total: ProgressBar,
    pb_current: ProgressBar,
}

impl UploadMultiProgress {
    fn new(parts: u64, total: u64) -> Self {
        let mp = MultiProgress::new();

        let pb_parts = mp.add(ProgressBar::new(parts))
            .with_prefix("Parts")
            .with_style(
		ProgressStyle::default_bar()
                    .template("{prefix:8} {wide_bar:40.cyan/blue} {pos}/{len}")
                    .unwrap()
                    .progress_chars("##-"),
            );
        pb_parts.set_position(0);

        let pb_total = mp.add(ProgressBar::new(total))
            .with_prefix("Total")
            .with_style(
		ProgressStyle::default_bar()
                    .template("{prefix:8} {wide_bar:40.green/black} {bytes}/{total_bytes} \
			       {bytes_per_sec} elapsed {elapsed} ETA {eta}")
                    .unwrap(),
            );

        Self { mp, pb_parts, pb_total }
    }

    fn new_part(&self, part_idx: usize, part_size: u64) -> UploadProgress {
        let pb_current = self.mp.add(ProgressBar::new(part_size))
            .with_prefix(format!("#{}", part_idx+1))
            .with_style(
		ProgressStyle::default_bar()
                    .template("{prefix:8} {wide_bar:40.yellow/black} {bytes}/{total_bytes} \
			       {bytes_per_sec}")
                .unwrap(),
            )
            .with_position(0);

        UploadProgress {
	    mp: self.mp.clone(),
	    pb_parts: self.pb_parts.clone(),
            pb_total: self.pb_total.clone(),
	    pb_current
        }
    }

    fn hide(&self) {
	self.mp.set_draw_target(ProgressDrawTarget::hidden())
    }

    fn println<S: AsRef<str>>(&self, msg: S) -> io::Result<()> {
	if self.mp.is_hidden() {
	    println!("{}", msg.as_ref());
	    Ok(())
	} else {
	    self.mp.println(msg)
	}
    }

    fn finish(&self) {
        self.pb_parts.finish();
        self.pb_total.finish();
    }
}

impl UploadProgress {
    fn wrap_read<R: Read>(&self, read: R) -> UploadProgressReadWrapper<R> {
	UploadProgressReadWrapper {
	    read,
	    pb_total: self.pb_total.clone(),
	    pb_current: self.pb_current.clone(),
	}
    }

    fn finish(&self) {
	self.pb_current.finish();
	self.pb_parts.inc(1);
	self.mp.remove(&self.pb_current);
    }

    fn reset(&self) {
	self.pb_current.reset();
    }
}

impl<R: Read> Read for UploadProgressReadWrapper<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let bytes_read = self.read.read(buf)?;
        self.pb_current.inc(bytes_read as u64);
        self.pb_total.inc(bytes_read as u64);
        Ok(bytes_read)
    }
}

fn do_upload_part(
    uploader: &s3::Uploader,
    path: &Path,
    url: &str,
    offset: u64,
    size: u64,
    pb: &UploadProgress,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut f = File::open(path)?;
    f.seek(SeekFrom::Start(offset))?;

    let part_reader = f.take(size);
    let buf_reader = BufReader::new(part_reader);

    let req = uploader.url(url)
	.mimetype("video/mp4")
	.from_reader_sized(pb.wrap_read(buf_reader), size);

    let res = req.upload()?;

    Ok(res.etag)
}

fn upload_part(
    uploader: &s3::Uploader,
    path: &Path,
    url: &str,
    offset: u64,
    size: u64,
    pb: UploadProgress,
    retry: u64,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut retry_count = 0;
    loop {
        let result = do_upload_part(uploader, path, url, offset, size, &pb);
        if result.is_ok() {
	    pb.finish();
            return result;
        }

        retry_count += 1;
        if retry_count >= retry {
            return result;
        }
	pb.reset();
    }
}

fn do_upload(
    uuid: &str,
    path: &Path,
    part_size: u64,
    no_progress: bool,
    retry: u64,
    hash: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let f = File::open(path)?;
    let f_size = f.metadata()?.len();

    std::println!("Initiating multi-part upload");
    let upload_start = api::video::upload_start(uuid, f_size, part_size, hash.clone())?;
    print_video_info(&upload_start.video);

    let parts = upload_start.urls.len() as u64;

    let mp = UploadMultiProgress::new(parts, f_size);
    if no_progress {
	mp.hide();
    }

    let uploader = s3::Uploader::new()?;

    let etags = upload_start.urls.par_iter().enumerate().map(|(i, url)| {
        let offset = (i as u64) * part_size;
        let size = min(part_size, f_size - offset);
	let pb = mp.new_part(i, part_size);
        let etag = upload_part(&uploader, path, url, offset, size, pb, retry)
            .expect(format!("Failed to uploading part {}", i+1).as_str());
        mp.println(format!("Part {} uploaded", i + 1)).unwrap();
        etag
    }).collect();

    api::video::upload_finish(uuid, upload_start.upload_id, etags, hash)?;
    mp.finish();

    Ok(())
}

pub(crate) fn main(args: Args) {
    std::println!("Uploading video file {path}", path = args.path.display());

    args.thread_count.map(|tc| {
        println!("Setting thread count to {tc}");
        rayon::ThreadPoolBuilder::new()
            .num_threads(tc)
            .build_global()
            .expect("Failed to set thread count")
    });

    let hash = args
        .password
        .map(|v| restricted_hash(&args.uuid, &v).unwrap());

    do_upload(
        &args.uuid,
        &args.path,
        args.part_size,
        args.no_progress,
        args.retry_part,
        hash,
    )
    .unwrap();
    println!("Upload finished")
}
