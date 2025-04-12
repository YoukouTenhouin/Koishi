use clap::Parser;
use std::path::PathBuf;

use crate::api;

#[derive(Parser)]
pub(super) struct Args {
    uuid: String,
    path: PathBuf,
}

pub(crate) fn main(args: Args) {
    std::println!("Uploading metadata file {path}", path = args.path.display());

    api::video::upload_metadata(&args.uuid, &args.path).unwrap();
}
