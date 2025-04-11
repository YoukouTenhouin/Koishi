use clap::Parser;
use std::path::PathBuf;

use crate::api;

#[derive(Parser)]
pub(super) struct Args {
    uuid: String,
    path: PathBuf,
}


pub(crate) fn main(args: Args) {
    let res = api::cover::upload_cover_from_file(args.path).unwrap();
    if res.exists {
	println!("Cover already presented in remote; skipping")
    } else {
	println!("Cover {} uploaded", res.hash)
    }

    crate::api::video::update(&args.uuid, None, Some(res.hash), None).unwrap()
}
