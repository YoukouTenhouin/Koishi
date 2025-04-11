use clap::Parser;
use tabled::{settings::{location::ByColumnName, Remove, Style}, Table};

use crate::api;

#[derive(Parser)]
pub(super) struct Args {
    #[arg(short, long)]
    cover: bool,
    #[arg(short, long, default_value_t=50)]
    limit: u64,
    #[arg(short, long, default_value_t=0)]
    offset: u64,

    room: u64,
}

pub(super) fn main(args: Args) {
    let res = api::room::list_videos(args.room, args.limit, args.offset);

    if let Err(e) = res {
	eprintln!("API Request Error: {e}");
	std::process::exit(-1)
    }

    let mut table = Table::new(res.unwrap());
    table.with(Style::modern());
    if !args.cover {
	table.with(Remove::column(ByColumnName::new("Cover")));
    }

    println!("{}", table);
}
