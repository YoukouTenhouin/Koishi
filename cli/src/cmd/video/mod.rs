use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use tabled::{Tabled, derive::display};

mod create;
mod import_from_xml;
mod restrict;
mod set_cover;
mod set_metadata;
mod upload;

#[derive(Parser)]
pub(crate) struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Serialize, Deserialize, Tabled)]
struct VideoInfoWithoutID {
    #[tabled(rename = "Title")]
    title: String,
    #[tabled(rename = "Cover URL", display("display::option", ""))]
    cover: Option<String>,
    #[tabled(rename = "Room ID")]
    room: u64,
    #[tabled(rename = "Date", display("display_timestamp", self))]
    timestamp: i64,
}

fn display_timestamp<T>(ts: &i64, _rec: &T) -> String {
    let date: DateTime<Utc> = DateTime::from_timestamp_millis(ts.to_owned()).unwrap();
    date.to_string()
}

#[derive(Subcommand)]
enum Commands {
    Create(create::Args),
    ImportFromXml(import_from_xml::Args),
    Restrict(restrict::Args),
    SetCover(set_cover::Args),
    SetMetadata(set_metadata::Args),
    Unrestrict(restrict::Args),
    Upload(upload::Args),
}

pub(crate) fn main(args: Args) {
    match args.command {
        Some(command) => match command {
            Commands::Create(args) => create::main(args),
            Commands::ImportFromXml(args) => import_from_xml::main(args),
            Commands::Restrict(args) => restrict::main(args, true),
            Commands::SetCover(args) => set_cover::main(args),
            Commands::SetMetadata(args) => set_metadata::main(args),
            Commands::Unrestrict(args) => restrict::main(args, false),
            Commands::Upload(args) => upload::main(args),
        },
        None => {}
    }
}
