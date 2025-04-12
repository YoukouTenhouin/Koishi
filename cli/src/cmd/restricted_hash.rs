use clap::Parser;

use crate::helpers::cryptography::restricted_hash;

#[derive(Parser)]
pub(crate) struct Args {
    #[arg(short, long)]
    password: String,

    uuid: String,
}

pub(crate) fn main(args: Args) {
    let hash = restricted_hash(&args.uuid.to_ascii_lowercase(), &args.password.trim()).unwrap();
    println!("{}", hash)
}
