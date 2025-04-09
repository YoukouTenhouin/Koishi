use chrono::{DateTime, Utc};
use clap::Parser;
use serde::Serialize;
use uuid::Uuid;

use crate::api_req::{self, Result, APIResponse};

#[derive(Parser)]
pub(super) struct Args {
    #[arg(short, long)]
    uuid: Option<String>,

    #[arg(short, long)]
    title: String,

    #[arg(short, long)]
    date: String,

    #[arg(short, long)]
    cover: Option<String>,

    #[arg(short, long)]
    room: u64,
}

#[derive(Serialize)]
struct VideoCreateInfo {
    title: String,
    cover: Option<String>,
    timestamp: i64,
    room: u64
}

fn do_create(
    uuid: Option<String>,
    title: String,
    cover: Option<String>,
    timestamp: i64,
    room: u64
) -> Result<()> {
    let uuid = uuid.unwrap_or_else(|| Uuid::now_v7().as_simple().to_string());

    let video = VideoCreateInfo { title, cover, room, timestamp };

    let path = format!("video/{uuid}");
    let res = api_req::post(path).json(&video).send()?;
    let body: APIResponse = res.json()?;

    println!("Created video {uuid}");

    Ok(body.unwrap_or_error_out())
}

pub(super) fn main(args: Args) {
    let ts = args.date.parse::<DateTime<Utc>>();

    if let Err(e) = ts {
	eprintln!("Can not parse date value \"{}\": {}", args.date, e.to_string());
	std::process::exit(-1);
    }

    let result = do_create(
	args.uuid,
	args.title,
	args.cover,
	ts.unwrap().timestamp_millis(),
	args.room
    );

    if let Err(e) = result {
	eprintln!("API Request Error: {e}");
	std::process::exit(-1)
    }
}
