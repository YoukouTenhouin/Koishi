use chrono::{DateTime, Utc};
use clap::Parser;
use quick_xml::de::from_str;
use serde::{Serialize, Deserialize};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use uuid::Uuid;

use crate::api_req::{self, Result, APIResponse};

#[derive(Parser)]
pub(super) struct Args {
    #[arg(short, long)]
    uuid: Option<String>,

    path: PathBuf,
}

#[derive(Deserialize, Debug)]
struct XMLRoomMetadata {
    room_id: u32,
    room_title: String,
    live_start_time: String,
}

#[derive(Deserialize)]
struct XMLRoot {
    metadata: XMLRoomMetadata
}

#[derive(Serialize)]
struct VideoCreateInfo {
    title: String,
    cover: Option<String>,
    timestamp: i64,
    room: u32
}

fn do_create(
    uuid: Option<String>,
    title: String,
    cover: Option<String>,
    timestamp: i64,
    room: u32
) -> Result<()> {
    let uuid = uuid.unwrap_or_else(|| Uuid::now_v7().as_simple().to_string());

    let video = VideoCreateInfo { title, cover, room, timestamp };

    let path = format!("video/{uuid}");
    let res = api_req::post(path).json(&video).send()?;
    let body: APIResponse = res.json()?;

    println!("Created video {uuid}");

    Ok(body.unwrap_or_error_out())
}

fn read_xml(path: &Path) -> XMLRoomMetadata {
    let mut file = File::open(path).expect("File not found");
    let mut xml_data = String::new();
    file.read_to_string(&mut xml_data).expect("XML read failed");

    match from_str::<XMLRoot>(&xml_data) {
        Ok(root) => {
	    root.metadata
        }
        Err(e) => {
            eprintln!("Error deserializing XML: {}", e);
	    std::process::exit(-1)
        }
    }
}

pub(super) fn main(args: Args) {
    let metadata = read_xml(&args.path);
    let ts = metadata.live_start_time.parse::<DateTime<Utc>>()
	.expect("Failed to parse live_start_time");

    let result = do_create(
	args.uuid,
	metadata.room_title,
	None,
	ts.timestamp_millis(),
	metadata.room_id
    );

    if let Err(e) = result {
	eprintln!("API Request Error: {e}");
	std::process::exit(-1)
    }
}
