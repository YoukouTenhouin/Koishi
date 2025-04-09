use clap::{Parser, Subcommand};

mod api_req;
mod cmd;
mod global_options;

#[derive(Parser)]
pub(crate) struct Cli {
    #[arg(short='u', long, value_name="URL", default_value="http://localhost:8788/api/")]
    base_url: String,

    #[arg(short='k', long, value_name="KEY", default_value="")]
    auth_key: String,

    #[arg(long)]
    dry: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    GenId,
    Liveroom(cmd::liveroom::Args),
    Video(cmd::video::Args),
}

fn main() {
    env_logger::init();

    let cli = Cli::parse();

    global_options::BASE_URL.set(cli.base_url).unwrap();
    global_options::AUTH_KEY.set(cli.auth_key).unwrap();
    global_options::DRY.set(cli.dry).unwrap();

    match cli.command {
	Some(command) => match command {
	    Commands::GenId => cmd::gen_id::main(),
	    Commands::Liveroom(args) => cmd::liveroom::main(args),
	    Commands::Video(args) => cmd::video::main(args),
	}
	None => {}
    }
}
