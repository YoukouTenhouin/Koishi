use clap::{Parser, Subcommand};
use serde::{Serialize, Deserialize};
use tabled::{Tabled, derive::display};

mod create;
mod get;
mod list;

#[derive(Parser)]
pub(crate) struct Args {
    #[command(subcommand)]
    command: Option<Commands>
}

#[derive(Serialize, Deserialize, Tabled)]
struct RoomInfo {
    #[tabled(rename="Room ID")]
    id: u32,
    #[tabled(rename="Short ID", display("display::option", ""))]
    short_id: Option<u32>,
    #[tabled(rename="Username")]
    username: String,
    #[tabled(rename="Image URL")]
    image: String,
}

#[derive(Subcommand)]
enum Commands{
    Create(create::Args),
    Get(get::Args),
    List
}

pub(crate) fn main(args: Args) {
    match args.command {
	Some(command) => match command {
	    Commands::Create(args) => create::main(args),
	    Commands::Get(args) => get::main(args),
	    Commands::List => list::main(),
	}
	None => {}
    }
}
