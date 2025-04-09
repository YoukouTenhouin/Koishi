use serde::Deserialize;
use tabled::{Tabled, Table, settings::Style, derive::display};

use crate::api_req::{self, Result, APIResponseWithData};

#[derive(Deserialize, Tabled)]
struct RoomListEntry {
    #[tabled(rename="Room ID")]
    id: u32,
    #[tabled(rename="Short ID", display("display::option", ""))]
    short_id: Option<u32>,
    #[tabled(rename="Username")]
    username: String,
}

fn do_list() -> Result<Vec<RoomListEntry>> {
    let res: APIResponseWithData<Vec<RoomListEntry>> = api_req::get("room").send()?.json()?;
    Ok(res.unwrap_or_error_out())
}

pub(super) fn main() {
    let res = do_list();

    if let Err(e) = res {
	eprintln!("API Request Error: {e}");
	std::process::exit(-1)
    }

    let mut table = Table::new(res.unwrap());
    table.with(Style::modern());

    println!("{}", table);
}
