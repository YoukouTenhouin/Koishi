use serde::{Deserialize, Serialize};
use tabled::{Tabled, derive::display};

use crate::global_options;
use crate::helpers::{self, se::BoolAsInt};

use super::{
    Result,
    request::{self, *},
};

#[derive(Serialize, Deserialize, Tabled)]
pub(crate) struct Room {
    #[tabled(rename = "Room ID")]
    pub id: u64,
    #[tabled(rename = "Short ID", display("display::option", ""))]
    pub short_id: Option<u32>,
    #[tabled(rename = "Username")]
    pub username: String,
    #[tabled(rename = "Image URL")]
    pub image: String,
}

#[derive(Deserialize, Tabled)]
pub(crate) struct RoomListVideoEntry {
    #[tabled(rename = "UUID")]
    uuid: String,
    #[tabled(rename = "Title")]
    title: String,
    #[tabled(rename = "Cover", display("display::option", "<Not set>"))]
    cover: Option<String>,
    #[tabled(rename = "Stream Time", display("helpers::tabled::timestamp", self))]
    stream_time: i64,
    #[tabled(rename = "Restricted")]
    restricted: BoolAsInt,
}

pub(crate) fn create(room: Room) -> Result<()> {
    if global_options::DRY.get().unwrap().clone() {
        println!("skipping request due to being dry run");
        return Ok(());
    }

    request::post(format!("room/{}", room.id))
        .json(&room)
        .send()?
        .api_result()
}

pub(crate) fn get(id: u64) -> Result<Room> {
    request::get(format!("room/{id}")).send()?.api_result()
}

pub(crate) fn list(limit: u64, offset: u64) -> Result<Vec<Room>> {
    request::get("room")
        .limit_offset(limit, offset)
        .send()?
        .api_result()
}

pub(crate) fn list_videos(id: u64, limit: u64, offset: u64) -> Result<Vec<RoomListVideoEntry>> {
    request::get(format!("room/{id}/video"))
        .limit_offset(limit, offset)
        .send()?
        .api_result()
}
