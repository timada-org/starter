use clap::{arg, Command};
use std::process;

#[tokio::main]
async fn main() {
    let matches = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg_required_else_help(true)
        .subcommand(
            Command::new("migrate")
                .about("Migrate database schema to latest")
                .arg(arg!(-c --config <CONFIG>).required(false)),
        )
        .subcommand(
            Command::new("serve")
                .about("Start starter server using bin")
                .arg(arg!(-c --config <CONFIG>).required(false)),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("migrate", _sub_matches)) => {
            println!("Migration database....");
        }
        Some(("reset", _sub_matches)) => {
            println!("Reset database...");
        }
        Some(("serve", _sub_matches)) => {
            process::Command::new("starter-server")
                .stdout(process::Stdio::null())
                .output()
                .expect("failed to execute starter-server");
        }
        _ => unreachable!(),
    };
}