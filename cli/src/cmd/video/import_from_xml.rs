use chrono::{DateTime, Utc};
use clap::Parser;
use quick_xml::de::from_str;
use serde::Deserialize;
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};
use uuid::Uuid;

use crate::api;
use crate::helpers::cryptography::restricted_hash;

#[derive(Parser)]
pub(super) struct Args {
    #[arg(short, long)]
    uuid: Option<String>,
    #[arg(short, long)]
    cover: Option<String>,
    #[arg(short, long)]
    password: Option<String>,

    path: PathBuf,
}

#[derive(Deserialize, Debug)]
struct XMLRoomMetadata {
    room_id: u64,
    room_title: String,
    live_start_time: String,
}

#[derive(Deserialize)]
struct XMLRoot {
    metadata: XMLRoomMetadata
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
    let uuid = args.uuid.unwrap_or_else(|| Uuid::now_v7().as_simple().to_string());
    let metadata = read_xml(&args.path);
    let ts = metadata.live_start_time.parse::<DateTime<Utc>>()
	.expect("Failed to parse live_start_time");

    let restricted_hash = args.password.map(|v| restricted_hash(&uuid, &v).unwrap());

    api::video::create(
	&uuid,
	metadata.room_title,
	args.cover,
	ts.timestamp_millis(),
	metadata.room_id,
	restricted_hash
    ).unwrap();

    println!("Created video {uuid} from XML");
}
