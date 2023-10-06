use crate::proto::tlcs::v1beta1::{
    QueryAllContributionsResponse, QueryAllKeyPairsResponse, QueryAllLoeDataResponse,
    QueryRoundRequest, QueryRoundSchemeRequest, QueryTimeRequest,
};
use anyhow::Result;
use clap::{Args, Subcommand};
use gears::client::query::run_query;
use ibc_proto::protobuf::Protobuf;

use tendermint_informal::block::Height;

#[derive(Args, Debug)]
pub struct QueryCli {
    #[command(subcommand)]
    command: TimelockCommands,
}

#[derive(Subcommand, Debug)]
pub enum TimelockCommands {
    /// Query list of all contributions
    Contributions,
    /// Query for contributions by round
    ContributionsByRound { round: u64 },
    /// Query for contributions by round and scheme
    ContributionsByRoundAndScheme { round: u64, scheme: u32 },
    /// Query list of all keypairs
    Keypairs,
    /// Query for keypairs by round
    KeypairsByRound { round: u64 },
    /// Query for keypairs by time
    KeypairsByTime { time: i64 },
    /// Query for keypairs by round and scheme
    KeypairsByRoundAndScheme { round: u64, scheme: u32 },
    /// Query list of all LOE data
    LoeData,
    /// Query for LOE data by round
    LoeDataByRound { round: u64 },
    /// Query list of keypairs that need loe data
    LoeDataNeeded,
}

pub fn run_timelock_query_command(
    args: QueryCli,
    node: &str,
    height: Option<Height>,
) -> Result<String> {
    match args.command {
        TimelockCommands::Contributions => {
            let res = run_query::<QueryAllContributionsResponse, QueryAllContributionsResponse>(
                vec![],
                "/tlcs.timelock.v1beta1.Query/AllContributions".into(),
                node,
                height,
            )?;

            Ok(serde_json::to_string_pretty(&res)?)
        }
        TimelockCommands::ContributionsByRound { round } => {
            let query = QueryRoundRequest { round };

            let res = run_query::<QueryAllContributionsResponse, QueryAllContributionsResponse>(
                query.encode_vec(),
                "/tlcs.timelock.v1beta1.Query/AllContributionsByRound".into(),
                node,
                height,
            )?;

            Ok(serde_json::to_string_pretty(&res)?)
        }
        TimelockCommands::ContributionsByRoundAndScheme { round, scheme } => {
            let query = QueryRoundSchemeRequest { round, scheme };

            let res = run_query::<QueryAllContributionsResponse, QueryAllContributionsResponse>(
                query.encode_vec(),
                "/tlcs.timelock.v1beta1.Query/AllContributionsByRoundAndScheme".into(),
                node,
                height,
            )?;

            Ok(serde_json::to_string_pretty(&res)?)
        }
        TimelockCommands::Keypairs => {
            let res = run_query::<QueryAllKeyPairsResponse, QueryAllKeyPairsResponse>(
                vec![],
                "/tlcs.timelock.v1beta1.Query/AllKeyPairs".into(),
                node,
                height,
            )?;

            Ok(serde_json::to_string_pretty(&res)?)
        }
        TimelockCommands::KeypairsByRound { round } => {
            let query = QueryRoundRequest { round };

            let res = run_query::<QueryAllKeyPairsResponse, QueryAllKeyPairsResponse>(
                query.encode_vec(),
                "/tlcs.timelock.v1beta1.Query/AllKeyPairsByRound".into(),
                node,
                height,
            )?;

            Ok(serde_json::to_string_pretty(&res)?)
        }
        TimelockCommands::KeypairsByTime { time } => {
            let query = QueryTimeRequest { time };

            let res = run_query::<QueryAllKeyPairsResponse, QueryAllKeyPairsResponse>(
                query.encode_vec(),
                "/tlcs.timelock.v1beta1.Query/AllKeyPairsByTime".into(),
                node,
                height,
            )?;

            Ok(serde_json::to_string_pretty(&res)?)
        }
        TimelockCommands::KeypairsByRoundAndScheme { round, scheme } => {
            let query = QueryRoundSchemeRequest { round, scheme };

            let res = run_query::<QueryAllKeyPairsResponse, QueryAllKeyPairsResponse>(
                query.encode_vec(),
                "/tlcs.timelock.v1beta1.Query/AllKeyPairsByRoundAndScheme".into(),
                node,
                height,
            )?;

            Ok(serde_json::to_string_pretty(&res)?)
        }
        TimelockCommands::LoeData => {
            let res = run_query::<QueryAllLoeDataResponse, QueryAllLoeDataResponse>(
                vec![],
                "/tlcs.timelock.v1beta1.Query/AllLoeData".into(),
                node,
                height,
            )?;

            Ok(serde_json::to_string_pretty(&res)?)
        }
        TimelockCommands::LoeDataByRound { round } => {
            let query = QueryRoundRequest { round };

            let res = run_query::<QueryAllLoeDataResponse, QueryAllLoeDataResponse>(
                query.encode_vec(),
                "/tlcs.timelock.v1beta1.Query/AllLoeDataByRound".into(),
                node,
                height,
            )?;

            Ok(serde_json::to_string_pretty(&res)?)
        }
        TimelockCommands::LoeDataNeeded => {
            let res = run_query::<QueryAllKeyPairsResponse, QueryAllKeyPairsResponse>(
                vec![],
                "/tlcs.timelock.v1beta1.Query/AllLoeDataNeeded".into(),
                node,
                height,
            )?;

            Ok(serde_json::to_string_pretty(&res)?)
        }
    }
}
