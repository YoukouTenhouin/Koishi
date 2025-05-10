use clap::Parser;
use tabled::{
    Table,
    settings::{Remove, Style, location::ByColumnName},
};

use crate::api;

#[derive(Parser)]
pub(super) struct Args {
    uuid: String,

    #[arg(short, long)]
    cover: bool,
}

pub(super) fn main(args: Args) {
    let video = api::video::get(&args.uuid).unwrap();

    let mut table = Table::new([video]);
    table.with(Style::modern());
    if !args.cover {
	table.with(Remove::column(ByColumnName::new("Cover URL")));
    }

    println!("{}", table);
}
