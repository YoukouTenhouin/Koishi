use chrono::{DateTime, Utc};
use clap::Parser;
use uuid::Uuid;

use crate::api;
use crate::helpers::cryptography::restricted_hash;

#[derive(Parser)]
pub(super) struct Args {
    #[arg(short, long)]
    uuid: Option<String>,

    #[arg(short, long)]
    title: String,

    #[arg(short, long)]
    date: String,

    #[arg(short, long)]
    cover: Option<String>,

    #[arg(short, long)]
    password: Option<String>,

    #[arg(short, long)]
    room: u64,
}

pub(super) fn main(args: Args) {
    let uuid = args
        .uuid
        .unwrap_or_else(|| Uuid::now_v7().as_simple().to_string());

    let ts = args.date.parse::<DateTime<Utc>>();
    if let Err(e) = ts {
        eprintln!(
            "Can not parse date value \"{}\": {}",
            args.date,
            e.to_string()
        );
        std::process::exit(-1);
    }

    let restricted_hash = args.password.map(|v| restricted_hash(&uuid, &v).unwrap());

    api::video::create(
        &uuid,
        args.title,
        args.cover,
        ts.unwrap().timestamp_millis(),
        args.room,
        restricted_hash,
    )
    .unwrap();

    println!("Created video {uuid}");
}
