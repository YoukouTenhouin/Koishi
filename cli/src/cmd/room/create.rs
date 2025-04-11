use clap::Parser;
use serde::Deserialize;
use serde_json::Value;
use reqwest::Url;
use reqwest::blocking as req;

use crate::api;

#[derive(Parser)]
pub(super) struct Args {
    room_id: u64,
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
    uid: u64,
    room_id: u64,
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

fn get_room_info(room_id: u64) -> Option<ResRoomInfo> {
    let url = Url::parse_with_params(ROOM_INFO_URL, &[("room_id", room_id.to_string())]).unwrap();
    let res: ResBody = req::get(url).unwrap().json().unwrap();
    if res.code == 1 {
	None
    } else {
	let info: ResRoomInfo = serde_json::value::from_value(res.data).unwrap();
	Some(info)
    }
}

fn get_user_info(uid: u64) -> ResUserInfo {
    let url = Url::parse_with_params(USER_INFO_URL, &[("mid", uid.to_string())]).unwrap();
    let res: ResBody = req::get(url).unwrap().json().unwrap();

    serde_json::value::from_value(res.data).unwrap()
}

fn fetch_info(room_id: u64) -> Option<api::room::Room> {
    let room_info = get_room_info(room_id)?;
    let short_id = if room_info.short_id == 0 {
	None
    } else {
	Some(room_info.short_id)
    };

    let user_info = get_user_info(room_info.uid);

    let ret = api::room::Room {
	id: room_info.room_id,
	short_id,
	username: user_info.card.name,
	image: user_info.card.face,
    };
    Some(ret)
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

    let id = info.id;
    println!("Creating room {}", info.id);
    api::room::create(info).unwrap();
    println!("Room {} created", id);
}
