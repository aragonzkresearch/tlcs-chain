use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::{arg, value_parser, ArgAction, ArgMatches, Command};

use crate::{
    utils::get_default_home_dir,
    x::{
        bank::client::cli::tx::{get_bank_tx_command, run_bank_tx_command},
        tlcs::client::cli::tx::{get_tlcs_tx_command, run_tlcs_tx_command},
    },
};

pub fn run_tx_command(matches: &ArgMatches) -> Result<()> {
    let node = matches
        .get_one::<String>("node")
        .expect("Node arg has a default value so this cannot be `None`.");

    let default_home_directory = get_default_home_dir();
    let home = matches
        .get_one::<PathBuf>("home")
        .or(default_home_directory.as_ref())
        .ok_or(anyhow!(
            "Home argument not provided and OS does not provide a default home directory"
        ))?
        .to_owned();

    match matches.subcommand() {
        Some(("bank", sub_matches)) => run_bank_tx_command(sub_matches, node, home),
        Some(("tlcs", sub_matches)) => run_tlcs_tx_command(sub_matches, node, home),
        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    }
}

pub fn get_tx_command() -> Command {
    Command::new("tx")
        .about("Transaction subcommands")
        .subcommand(get_bank_tx_command())
        .subcommand(get_tlcs_tx_command())
        .subcommand_required(true)
        .arg(
            arg!(--node)
                .help("<host>:<port> to Tendermint RPC interface for this chain")
                .default_value("http://localhost:26657")
                .action(ArgAction::Set)
                .global(true),
        )
        .arg(
            arg!(--home)
                .help(format!(
                    "Directory for config and data [default: {}]",
                    get_default_home_dir()
                        .unwrap_or_default()
                        .display()
                        .to_string()
                ))
                .action(ArgAction::Set)
                .value_parser(value_parser!(PathBuf)),
        )
}
