use anyhow::Result;
use auth::cli::query::{run_auth_query_command, QueryCli as AuthQueryCli};
use bank::cli::{
    query::{run_bank_query_command, QueryCli as BankQueryCli},
    tx::{run_bank_tx_command, Cli as BankCli},
};
use clap::Subcommand;
use proto_types::AccAddress;
use tendermint_informal::block::Height;
use timelock::cli::{
    query::{run_timelock_query_command, QueryCli as TimelockQueryCli},
    tx::{run_timelock_tx_command, Cli as TimelockCli},
};

use crate::message::Message;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Bank transaction subcommands
    Bank(BankCli),
    /// Timelock transaction subcommands
    Timelock(TimelockCli),
}

pub fn tx_command_handler(command: Commands, from_address: AccAddress) -> Result<Message> {
    match command {
        Commands::Bank(args) => run_bank_tx_command(args, from_address).map(Message::Bank),
        Commands::Timelock(args) => {
            run_timelock_tx_command(args, from_address).map(Message::Timelock)
        }
    }
    //run_bank_tx_command(args, from_address).map(|msg| Message::Bank(msg))
    // Clippy suggested this change
    //run_timelock_tx_command(args, from_address).map(|msg| Message::Timelock(msg))
    // Clippy suggested this change
}

#[derive(Subcommand, Debug)]
pub enum QueryCommands {
    /// Querying commands for the bank module
    Bank(BankQueryCli),
    /// Querying commands for the auth module
    Auth(AuthQueryCli),
    /// Querying commands for the timelock module
    Timelock(TimelockQueryCli),
}

pub fn query_command_handler(
    command: QueryCommands,
    node: &str,
    height: Option<Height>,
) -> Result<()> {
    let res = match command {
        QueryCommands::Bank(args) => run_bank_query_command(args, node, height),
        QueryCommands::Auth(args) => run_auth_query_command(args, node, height),
        QueryCommands::Timelock(args) => run_timelock_query_command(args, node, height),
    }?;

    println!("{}", res);
    Ok(())
}
