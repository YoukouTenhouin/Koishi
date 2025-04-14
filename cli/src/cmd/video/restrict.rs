use clap::Parser;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use rayon::prelude::*;
use std::cmp::min;

use crate::api;
use crate::helpers::{cryptography::restricted_hash, s3};

#[derive(Parser)]
pub(super) struct Args {
    #[arg(short = 'P', long)]
    no_progress: bool,
    #[arg(short, long)]
    password: String,
    #[arg(short = 's', long, default_value_t = 100_000_000)]
    part_size: u64,

    #[arg(short, long)]
    thread_count: Option<usize>,

    uuid: String,
}

pub(super) fn main(args: Args, restricted: bool) {
    let hash = restricted_hash(&args.uuid, &args.password).unwrap();

    println!("Updating restricted state");
    let ret = api::video::set_restricted(&args.uuid, restricted, &hash).unwrap();

    if ret.copy_source.is_none() {
        println!("Video not uploaded yet; skipped renaming");
        return;
    }
    let source = ret.copy_source.unwrap();

    let hash = if restricted { Some(hash) } else { None };

    std::println!("Intiating multi-part copy");
    let copy_start = api::video::restricted_copy_start(
        &args.uuid,
        source.as_str(),
        hash.clone(),
        args.part_size,
    )
    .expect("Failed to initiate multi-part copy");

    args.thread_count.map(|tc| {
        println!("Setting thread count to {tc}");
        rayon::ThreadPoolBuilder::new()
            .num_threads(tc)
            .build_global()
            .expect("Failed to set thread count")
    });

    let parts = copy_start.urls.len() as u64;

    let pb = ProgressBar::new(parts);
    if args.no_progress {
        pb.set_draw_target(ProgressDrawTarget::hidden());
    }

    pb.set_style(
        ProgressStyle::default_bar()
            .template("Parts   {bar:40.cyan/blue} {pos}/{len}")
            .unwrap()
            .progress_chars("##-"),
    );
    pb.set_position(0);

    let uploader = s3::Uploader::new().expect("Failed to create S3 Uploader");

    let etags = copy_start
        .urls
        .par_iter()
        .enumerate()
        .map(|(i, url)| {
            let range_from = (i as u64) * args.part_size;
            let range_to = min(range_from + args.part_size, copy_start.length) - 1;
            let etag = uploader
                .url(url)
                .mimetype("video/mp4")
                .copy(&source)
                .copy_range_from_to(range_from, range_to)
                .upload()
                .expect("S3 upload error")
                .etag;

            pb.inc(1);
            etag
        })
        .collect();

    pb.finish();
    println!("Multi-part copy finished");

    api::video::restricted_copy_finish(&args.uuid, &source, hash, &copy_start.upload_id, etags)
        .expect("Failed to finish upload");

    println!("Completed")
}
