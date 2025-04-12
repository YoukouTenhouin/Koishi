use clap::Parser;
use tabled::{
    Table,
    settings::{Remove, Style, location::ByColumnName},
};

use crate::api;

#[derive(Parser)]
pub(super) struct Args {
    #[arg(short, long)]
    image: bool,
    room: u64,
}

pub(super) fn main(args: Args) {
    let room = api::room::get(args.room).unwrap();

    let mut table = Table::new(&[room]);
    table.with(Style::modern());
    if !args.image {
        table.with(Remove::column(ByColumnName::new("Image")));
    }
    print!("{}", table)
}
