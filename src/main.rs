use colored::Colorize;
use dialoguer::Confirm;
use plan::Plan;
use std::error::Error;
use std::process::exit;

use crate::local::Root;
use crate::remote::Job as RemoteJob;
use crate::{config::Config, remote::DbtCloudClient};

mod config;
mod diff;
mod local;
mod plan;
mod remote;
mod utils;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help = true)]
struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    /// Commands
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Plans the changes derived from your dbt_cloud.yml file
    Plan,
    /// Plans and applies the changes derived from your dbt_cloud.yml file
    Apply {
        #[arg(short, long, default_value_t = false)]
        auto_approve: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.debug {
        0 => {}
        1 => println!("Debug mode is kind of on"),
        2 => println!("Debug mode is on"),
        _ => println!("Don't be crazy"),
    }

    // TODO: Move into function and call from correct commands
    let parse_yaml = read_yaml_file("./dbt_cloud.yml");
    if parse_yaml.is_err() {
        println!("{}", "failed to read dbt_cloud.yml file:".red());
        println!("  {}", parse_yaml.err().unwrap());
        exit(1);
    }

    let yaml = parse_yaml.unwrap();
    let config = Config::build(&yaml).expect("failed to build config");
    let client = DbtCloudClient::new(&config);

    match &cli.command {
        Some(Commands::Plan) => {
            let plan = Plan::from(yaml, &client, &config);
            plan.pretty_print();
            println!("\nno changes applied. to apply changes, run `dbt-cloud-sync apply`");
        }
        Some(Commands::Apply { auto_approve }) => {
            let plan = Plan::from(yaml, &client, &config);
            if !plan.has_changes() {
                println!(
                    "{}",
                    "no changes detected for any project, exiting...".red()
                );
                exit(0);
            }
            plan.pretty_print();
            println!();
            let mut prompt = Confirm::new();
            let prompt = prompt
                .with_prompt("do you want to apply the above changes?")
                .wait_for_newline(true);
            if *auto_approve || prompt.interact().unwrap() {
                println!("applying changes...");
                plan.apply(&client);
            }
        }
        None => {}
    }
}

fn read_yaml_file(filename: &str) -> Result<Root, Box<dyn Error>> {
    let f = std::fs::File::open(filename)?;
    let d = serde_yaml::from_reader(f)?;
    Ok(d)
}
