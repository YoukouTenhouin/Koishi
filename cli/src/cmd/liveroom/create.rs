use clap::Parser;
use serde::Deserialize;
use serde_json::Value;
use reqwest::Url;
use reqwest::blocking as req;

use crate::api_req::{self, APIResponse};
use crate::global_options;

use super::RoomInfo;

#[derive(Parser)]
pub(super) struct Args {
    room_id: u32,
}

const ROOM_INFO_URL: &str = "https://api.live.bilibili.com/room/v1/Room/get_info";
const USER_INFO_URL: &str = "https://api.bilibili.com/x/web-interface/card";

#[derive(Deserialize)]
struct ResBody {
    code: i32,
    data: Value,
}

#[derive(Deserialize)]
struct ResRoomInfo {
    uid: u32,
    room_id: u32,
    short_id: u32,
}

#[derive(Deserialize)]
struct ResUserInfo {
    card: ResUserCardInfo,
}

#[derive(Deserialize)]
struct ResUserCardInfo {
    name: String,
    face: String,
}

fn get_room_info(room_id: u32) -> Option<ResRoomInfo> {
    let url = Url::parse_with_params(ROOM_INFO_URL, &[("room_id", room_id.to_string())]).unwrap();
    let res: ResBody = req::get(url).unwrap().json().unwrap();
    if res.code == 1 {
	None
    } else {
	let info: ResRoomInfo = serde_json::value::from_value(res.data).unwrap();
	Some(info)
    }
}

fn get_user_info(uid: u32) -> ResUserInfo {
    let url = Url::parse_with_params(USER_INFO_URL, &[("mid", uid.to_string())]).unwrap();
    let res: ResBody = req::get(url).unwrap().json().unwrap();

    serde_json::value::from_value(res.data).unwrap()
}

fn fetch_info(room_id: u32) -> Option<RoomInfo> {
    let room_info = get_room_info(room_id)?;
    let short_id = if room_info.short_id == 0 {
	None
    } else {
	Some(room_info.short_id)
    };

    let user_info = get_user_info(room_info.uid);

    let ret = RoomInfo {
	id: room_info.room_id,
	short_id,
	username: user_info.card.name,
	image: user_info.card.face,
    };
    Some(ret)
}

fn do_create(info: &RoomInfo) -> api_req::Result<()> {
    println!("Creating room {}", info.id);
    if global_options::DRY.get().unwrap().clone() {
	println!("Skipping request due to being dry run");
	return Ok(())
    }

    let mut path = info.id.to_string();
    path.insert_str(0, "room/");
    let res: APIResponse = api_req::post(path)
        .json(info)
        .send()?
	.json()?;

    res.unwrap_or_error_out();

    println!("Room {} created", info.id);
    Ok(())
}

pub(super) fn main(args: Args) {
    println!("Fetching info for room {}", args.room_id);

    let Some(info) = fetch_info(args.room_id) else {
	println!("Room {} not found", args.room_id);
	return;
    };

    println!("\tID:\t\t{}", info.id);
    if let Some(short_id) = info.short_id {
	println!("\tShort ID:\t{}", short_id);
    }
    println!("\tName:\t\t{}", info.username);
    println!("\tImage:\t\t{}", info.image);

    if let Err(e) = do_create(&info) {
	eprintln!("API Request Error: {e}");
	std::process::exit(-1);
    }
}
