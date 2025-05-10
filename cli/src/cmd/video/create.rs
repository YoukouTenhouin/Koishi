use chrono::{DateTime, Utc};
use clap::Parser;
use uuid::Uuid;

use crate::api;
use crate::helpers::cryptography::restricted_hash;

#[derive(Parser)]
pub(super) struct Args {
    #[arg(short, long)]
    uuid: Option<String>,

    #[arg(short, long)]
    title: String,

    #[arg(long)]
    stream_time: String,

    #[arg(long)]
    record_time: String,

    #[arg(short, long)]
    cover: Option<String>,

    #[arg(short, long)]
    password: Option<String>,

    #[arg(short, long)]
    room: u64,
}

fn parse_timestamp(date_str: &str) -> i64 {
    date_str.parse::<DateTime<Utc>>()
        .expect(format!("Failed to parse timestamp {date_str}").as_str())
        .timestamp_millis()
}

pub(super) fn main(args: Args) {
    let uuid = args
        .uuid
        .unwrap_or_else(|| Uuid::now_v7().as_simple().to_string());

    let stream_time = parse_timestamp(&args.stream_time);
    let record_time = parse_timestamp(&args.record_time);

    let restricted_hash = args.password.map(|v| restricted_hash(&uuid, &v).unwrap());

    api::video::create(
        &uuid,
        args.title,
        args.cover,
	stream_time,
	record_time,
        args.room,
        restricted_hash,
    )
    .unwrap();

    println!("Created video {uuid}");
}
