use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use serde::{Serialize, Deserialize};
use tabled::{Tabled, derive::display};

mod create;
mod set_cover;
mod upload;

#[derive(Parser)]
pub(crate) struct Args {
    #[command(subcommand)]
    command: Option<Commands>
}

#[derive(Serialize, Deserialize, Tabled)]
struct VideoInfo {
    #[tabled(rename="UUID")]
    uuid: String,
    #[tabled(rename="Title")]
    title: String,
    #[tabled(rename="Cover URL", display("display::option", ""))]
    cover: Option<String>,
    #[tabled(rename="Room ID")]
    room: u32,
    #[tabled(rename="Date", display("display_timestamp", self))]
    timestamp: i64
}

#[derive(Serialize, Deserialize, Tabled)]
struct VideoInfoWithoutID {
    #[tabled(rename="Title")]
    title: String,
    #[tabled(rename="Cover URL", display("display::option", ""))]
    cover: Option<String>,
    #[tabled(rename="Room ID")]
    room: u32,
    #[tabled(rename="Date", display("display_timestamp", self))]
    timestamp: i64
}

fn display_timestamp<T>(ts: &i64, _rec: &T) -> String {
    let date: DateTime<Utc> = DateTime::from_timestamp_millis(ts.to_owned()).unwrap();
    date.to_string()
}

#[derive(Subcommand)]
enum Commands{
    Create(create::Args),
    SetCover(set_cover::Args),
    Upload(upload::Args),
}

pub(crate) fn main(args: Args) {
    match args.command {
	Some(command) => match command {
	    Commands::Create(args) => create::main(args),
	    Commands::SetCover(args) => set_cover::main(args),
	    Commands::Upload(args) => upload::main(args),
	}
	None => {}
    }
}

