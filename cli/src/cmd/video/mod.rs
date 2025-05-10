use clap::{Parser, Subcommand};

mod create;
mod get;
mod from_xml;
mod restrict;
mod set_cover;
mod set_metadata;
mod upload;

#[derive(Parser)]
pub(crate) struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Create(create::Args),
    Get(get::Args),
    ImportFromXml(from_xml::ImportArgs),
    Restrict(restrict::Args),
    SetCover(set_cover::Args),
    SetMetadata(set_metadata::Args),
    Unrestrict(restrict::Args),
    UpdateFromXml(from_xml::UpdateArgs),
    Upload(upload::Args),
}

pub(crate) fn main(args: Args) {
    match args.command {
        Some(command) => match command {
            Commands::Create(args) => create::main(args),
	    Commands::Get(args) => get::main(args),
            Commands::ImportFromXml(args) => from_xml::import(args),
            Commands::Restrict(args) => restrict::main(args, true),
            Commands::SetCover(args) => set_cover::main(args),
            Commands::SetMetadata(args) => set_metadata::main(args),
            Commands::Unrestrict(args) => restrict::main(args, false),
	    Commands::UpdateFromXml(args) => from_xml::update(args),
            Commands::Upload(args) => upload::main(args),
        },
        None => {}
    }
}
