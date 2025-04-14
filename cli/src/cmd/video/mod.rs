use clap::{Parser, Subcommand};

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
