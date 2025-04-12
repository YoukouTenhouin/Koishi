use clap::{Parser, Subcommand};

mod api;
mod cmd;
mod global_options;
mod helpers;

#[derive(Parser)]
pub(crate) struct Cli {
    #[arg(
        short = 'u',
        long,
        value_name = "URL",
        default_value = "http://localhost:8788/api/"
    )]
    base_url: String,

    #[arg(short = 'k', long, value_name = "KEY")]
    auth_key: Option<String>,

    #[arg(long)]
    dry: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    GenId,
    RestrictedHash(cmd::restricted_hash::Args),
    Room(cmd::room::Args),
    Video(cmd::video::Args),
}

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    let base_url = if cli.base_url.ends_with('/') {
        cli.base_url
    } else {
        let mut base_url = cli.base_url;
        base_url.push('/');
        base_url
    };

    global_options::BASE_URL.set(base_url).unwrap();
    global_options::AUTH_KEY.set(cli.auth_key).unwrap();
    global_options::DRY.set(cli.dry).unwrap();

    match cli.command {
        Some(command) => match command {
            Commands::GenId => cmd::gen_id::main(),
            Commands::RestrictedHash(args) => cmd::restricted_hash::main(args),
            Commands::Room(args) => cmd::room::main(args),
            Commands::Video(args) => cmd::video::main(args),
        },
        None => {}
    }
}
