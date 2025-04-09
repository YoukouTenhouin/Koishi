use chrono::{DateTime, Utc};
use clap::Parser;
use serde::Deserialize;
use tabled::{Tabled, Table, settings::Style};

use crate::api_req::{self, Result, APIResponseWithData};

#[derive(Parser)]
pub(super) struct Args {
    room: u32,
}

#[derive(Deserialize, Tabled)]
struct VideoListEntry {
    #[tabled(rename="UUID")]
    uuid: String,
    #[tabled(rename="Title")]
    title: String,
    #[tabled(rename="Username", display("display_timestamp", self))]
    timestamp: i64,
}

fn display_timestamp<T>(ts: &i64, _rec: &T) -> String {
    let date: DateTime<Utc> = DateTime::from_timestamp_millis(ts.to_owned()).unwrap();
    date.to_string()
}

fn do_list(room: u32) -> Result<Vec<VideoListEntry>> {
    let url = format!("room/{room}/video");
    let res: APIResponseWithData<Vec<VideoListEntry>> =
	api_req::get(url).send()?.json()?;
    Ok(res.unwrap_or_error_out())
}

pub(super) fn main(args: Args) {
    let res = do_list(args.room);

    if let Err(e) = res {
	eprintln!("API Request Error: {e}");
	std::process::exit(-1)
    }

    let mut table = Table::new(res.unwrap());
    table.with(Style::modern());

    println!("{}", table);
}
