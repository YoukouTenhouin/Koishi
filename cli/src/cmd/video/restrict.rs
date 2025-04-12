use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use std::cmp::min;

use crate::api;
use crate::helpers::{
    cryptography::restricted_hash,
    s3
};

#[derive(Parser)]
pub(super) struct Args {
    #[arg(short='P', long)]
    progress: bool,
    #[arg(short, long)]
    password: String,
    #[arg(short='s', long, default_value_t=3_000_000_000)]
    part_size: u64,
    #[arg(short, long, default_value_t=10)]
    retry_part: u64,

    uuid: String
}

pub(super) fn main(args: Args, restricted: bool) {
    let hash = restricted_hash(&args.uuid, &args.password).unwrap();

    println!("Updating restricted state");
    let ret = api::video::set_restricted(&args.uuid, restricted, &hash).unwrap();

    if ret.is_none() {
	println!("Video not uploaded yet; skipped renaming");
	return
    }
    let ret = ret.unwrap();

    let hash = if restricted { Some(hash) } else { None };

    std::println!("Intiating multi-part copy");
    let copy_start = api::video::restricted_copy_start(
	&args.uuid,
	ret.copy_source.clone(),
	hash.clone(),
	args.part_size
    ).expect("Failed to initiate multi-part copy");

    let parts = copy_start.urls.len() as u64;

    let pb: Option<ProgressBar> = if args.progress {
	let pb = ProgressBar::new(parts);
	pb.set_style(ProgressStyle::default_bar()
		     .template("Parts   {bar:40.cyan/blue} {pos}/{len}")
		     .unwrap()
		     .progress_chars("##-"));
	pb.set_position(0);
	Some(pb)
    } else {
	None
    };

    let uploader = s3::Uploader::new().expect("Failed to create s3 Uploader");

    let mut etags: Vec<String> = Vec::with_capacity(parts as usize);
    for (i, url) in copy_start.urls.iter().enumerate() {
	let range_from = (i as u64) * args.part_size;
	let range_to = min(
	    range_from + args.part_size,
	    copy_start.length
	) - 1;
	let etag = uploader
	    .url(url)
	    .mimetype("video/mp4")
	    .copy(&ret.copy_source)
	    .copy_range_from_to(range_from, range_to)
	    .upload()
	    .expect("S3 upload error")
	    .etag;

	etags.push(etag);
	pb.as_ref().map(|v| v.inc(1));
    }

    pb.map(|v| v.finish_with_message("Multi-part copy finished"))
        .or_else(|| Some(println!("Multi-part copy finished")));

    api::video::restricted_copy_finish(
	&args.uuid, ret.copy_source, hash, copy_start.upload_id, etags
    )
        .expect("Failed to finish upload");
}
