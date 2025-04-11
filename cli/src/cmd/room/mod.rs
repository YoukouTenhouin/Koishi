use clap::{Parser, Subcommand};

mod create;
mod get;
mod list;
mod list_videos;

#[derive(Parser)]
pub(crate) struct Args {
    #[command(subcommand)]
    command: Option<Commands>
}

#[derive(Subcommand)]
enum Commands{
    Create(create::Args),
    Get(get::Args),
    List(list::Args),
    ListVideos(list_videos::Args),
}

pub(crate) fn main(args: Args) {
    match args.command {
	Some(command) => match command {
	    Commands::Create(args) => create::main(args),
	    Commands::Get(args) => get::main(args),
	    Commands::List(args) => list::main(args),
	    Commands::ListVideos(args) => list_videos::main(args),
	}
	None => {}
    }
}
