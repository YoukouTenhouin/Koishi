use clap::Parser;
use tabled::{Table, settings::Style};

use crate::api_req::{self, APIResponseWithData, Result};

use super::RoomInfo;

#[derive(Parser)]
pub(super) struct Args {
    room: u32,
}


fn do_get(room_id: u32) -> Result<Option<RoomInfo>> {
    let mut path = room_id.to_string();
    path.insert_str(0, "room/");
    let res = api_req::get(path).send()?;
    let status = res.status();
    let body: APIResponseWithData<RoomInfo> = res.json()?;

    if status == 404 {
	Ok(None)
    } else {
	Ok(Some(body.unwrap_or_error_out()))
    }
}

pub(super) fn main(args: Args) {
    let res = do_get(args.room);

    if let Err(e) = res {
	eprintln!("API Request Error: {e}");
	std::process::exit(-1)
    }

    let info = res.unwrap();
    if info.is_none() {
	println!("Room {} not found", args.room);
	std::process::exit(255)
    }

    let info = info.unwrap();

    let mut table = Table::new(&[info]);
    table.with(Style::modern());
    print!("{}", table)
}
