use clap::Parser;
use tabled::{Table, settings::Style};

use crate::api;

#[derive(Parser)]
pub(super) struct Args {
    #[arg(short, long, default_value_t = 50)]
    limit: u64,
    #[arg(short, long, default_value_t = 0)]
    offset: u64,
}

pub(super) fn main(args: Args) {
    let res = api::room::list(args.limit, args.offset).unwrap();

    let mut table = Table::new(res);
    table.with(Style::modern());

    println!("{}", table);
}
