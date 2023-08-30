use crate::proto::tlcs::v1beta1::{
    QueryAllContributionsResponse, QueryAllKeyPairsResponse, QueryAllLoeDataResponse,
    QueryRoundRequest, QueryRoundSchemeRequest, QueryTimeRequest,
};
use anyhow::{anyhow, Result};
use clap::{Arg, ArgMatches, Command};
use ibc_proto::protobuf::Protobuf;
use prost::Message;

use tendermint_rpc::{Client, HttpClient};
use tokio::runtime::Runtime;

pub fn get_timelock_query_command() -> Command {
    Command::new("timelock")
        .about("Querying commands for the timelock module")
        .subcommand(Command::new("contributions").about("Query list of all contributions"))
        .subcommand(
            Command::new("contributions_by_round")
                .about("Query for contributions by round")
                .arg(
                    Arg::new("round")
                        .required(true)
                        .value_parser(clap::value_parser!(u64)),
                ),
        )
        .subcommand(
            Command::new("contributions_by_round_and_scheme")
                .about("Query for contributions by round and scheme")
                .arg(
                    Arg::new("round")
                        .required(true)
                        .value_parser(clap::value_parser!(u64)),
                )
                .arg(
                    Arg::new("scheme")
                        .required(true)
                        .value_parser(clap::value_parser!(u32)),
                ),
        )
        .subcommand(Command::new("keypairs").about("Query list of all keypairs"))
        .subcommand(
            Command::new("keypairs_by_round")
                .about("Query for keypairs by round")
                .arg(
                    Arg::new("round")
                        .required(true)
                        .value_parser(clap::value_parser!(u64)),
                ),
        )
        .subcommand(
            Command::new("keypairs_by_time")
                .about("Query for keypairs by time")
                .arg(
                    Arg::new("time")
                        .required(true)
                        .value_parser(clap::value_parser!(i64)),
                ),
        )
        .subcommand(
            Command::new("keypairs_by_round_and_scheme")
                .about("Query for keypairs by round and scheme")
                .arg(
                    Arg::new("round")
                        .required(true)
                        .value_parser(clap::value_parser!(u64)),
                )
                .arg(
                    Arg::new("scheme")
                        .required(true)
                        .value_parser(clap::value_parser!(u32)),
                ),
        )
        .subcommand(Command::new("loe_data").about("Query list of all LOE data"))
        .subcommand(
            Command::new("loe_data_by_round")
                .about("Query for LOE data by round")
                .arg(
                    Arg::new("round")
                        .required(true)
                        .value_parser(clap::value_parser!(u64)),
                ),
        )
        .subcommand_required(true)
}

pub fn run_timelock_query_command(matches: &ArgMatches, node: &str) -> Result<String> {
    let client = HttpClient::new(node)?;

    match matches.subcommand() {
        Some(("contributions", _sub_matches)) => Runtime::new()
            .expect("unclear why this would ever fail")
            .block_on(get_all_contributions(client)),
        Some(("contributions_by_round", sub_matches)) => {
            let round = sub_matches
                .get_one::<u64>("round")
                .expect("address argument is required preventing `None`")
                .to_owned();
            Runtime::new()
                .expect("unclear why this would ever fail")
                .block_on(get_contributions_by_round(client, round))
        }
        Some(("contributions_by_round_and_scheme", sub_matches)) => {
            let round = sub_matches
                .get_one::<u64>("round")
                .expect("address argument is required preventing `None`")
                .to_owned();
            let scheme = sub_matches
                .get_one::<u32>("scheme")
                .expect("address argument is required preventing `None`")
                .to_owned();
            Runtime::new()
                .expect("unclear why this would ever fail")
                .block_on(get_contributions_by_round_and_scheme(client, round, scheme))
        }
        Some(("keypairs", _sub_matches)) => Runtime::new()
            .expect("unclear why this would ever fail")
            .block_on(get_all_keypairs(client)),
        Some(("keypairs_by_round", sub_matches)) => {
            let round = sub_matches
                .get_one::<u64>("round")
                .expect("address argument is required preventing `None`")
                .to_owned();
            Runtime::new()
                .expect("unclear why this would ever fail")
                .block_on(get_keypairs_by_round(client, round))
        }
        Some(("keypairs_by_time", sub_matches)) => {
            let time = sub_matches
                .get_one::<i64>("time")
                .expect("address argument is required preventing `None`")
                .to_owned();
            Runtime::new()
                .expect("unclear why this would ever fail")
                .block_on(get_keypairs_by_time(client, time))
        }
        Some(("keypairs_by_round_and_scheme", sub_matches)) => {
            let round = sub_matches
                .get_one::<u64>("round")
                .expect("address argument is required preventing `None`")
                .to_owned();
            let scheme = sub_matches
                .get_one::<u32>("scheme")
                .expect("address argument is required preventing `None`")
                .to_owned();
            Runtime::new()
                .expect("unclear why this would ever fail")
                .block_on(get_keypairs_by_round_and_scheme(client, round, scheme))
        }
        Some(("loe_data", _sub_matches)) => Runtime::new()
            .expect("unclear why this would ever fail")
            .block_on(get_all_loe_data(client)),
        Some(("loe_data_by_round", sub_matches)) => {
            let round = sub_matches
                .get_one::<u64>("round")
                .expect("address argument is required preventing `None`")
                .to_owned();
            Runtime::new()
                .expect("unclear why this would ever fail")
                .block_on(get_loe_data_by_round(client, round))
        }
        _ => unreachable!("exhausted list of subcommands and subcommand_required prevents `None`"),
    }
}

pub async fn get_all_contributions(client: HttpClient) -> Result<String> {
    let res = client
        .abci_query(
            Some(
                "/azkr.tlcs.v1beta1.Query/AllContributions"
                    .parse()
                    .expect("hard coded path will always succeed"),
            ),
            vec![],
            None,
            false,
        )
        .await?;

    if res.code.is_err() {
        return Err(anyhow!("node returned an error: {}", res.log));
    }

    let res = QueryAllContributionsResponse::decode(&*res.value)?;
    Ok(serde_json::to_string_pretty(&res)?)
}

pub async fn get_contributions_by_round(client: HttpClient, round: u64) -> Result<String> {
    let query = QueryRoundRequest { round };
    let res = client
        .abci_query(
            Some(
                "/azkr.tlcs.v1beta1.Query/AllContributionsByRound"
                    .parse()
                    .expect("hard coded path will always succeed"),
            ),
            query.encode_vec(),
            None,
            false,
        )
        .await?;

    if res.code.is_err() {
        return Err(anyhow!("node returned an error: {}", res.log));
    }

    let res = QueryAllContributionsResponse::decode(&*res.value)?;
    Ok(serde_json::to_string_pretty(&res)?)
}

pub async fn get_contributions_by_round_and_scheme(
    client: HttpClient,
    round: u64,
    scheme: u32,
) -> Result<String> {
    let query = QueryRoundSchemeRequest { round, scheme };
    let res = client
        .abci_query(
            Some(
                "/azkr.tlcs.v1beta1.Query/AllContributionsByRoundAndScheme"
                    .parse()
                    .expect("hard coded path will always succeed"),
            ),
            query.encode_vec(),
            None,
            false,
        )
        .await?;

    if res.code.is_err() {
        return Err(anyhow!("node returned an error: {}", res.log));
    }

    let res = QueryAllContributionsResponse::decode(&*res.value)?;
    Ok(serde_json::to_string_pretty(&res)?)
}

pub async fn get_all_keypairs(client: HttpClient) -> Result<String> {
    let res = client
        .abci_query(
            Some(
                "/azkr.tlcs.v1beta1.Query/AllKeyPairs"
                    .parse()
                    .expect("hard coded path will always succeed"),
            ),
            vec![],
            None,
            false,
        )
        .await?;

    if res.code.is_err() {
        return Err(anyhow!("node returned an error: {}", res.log));
    }

    let res = QueryAllKeyPairsResponse::decode(&*res.value)?;
    Ok(serde_json::to_string_pretty(&res)?)
}

pub async fn get_keypairs_by_round(client: HttpClient, round: u64) -> Result<String> {
    let query = QueryRoundRequest { round };
    let res = client
        .abci_query(
            Some(
                "/azkr.tlcs.v1beta1.Query/AllKeyPairsByRound"
                    .parse()
                    .expect("hard coded path will always succeed"),
            ),
            query.encode_vec(),
            None,
            false,
        )
        .await?;

    if res.code.is_err() {
        return Err(anyhow!("node returned an error: {}", res.log));
    }

    let res = QueryAllKeyPairsResponse::decode(&*res.value)?;
    Ok(serde_json::to_string_pretty(&res)?)
}

pub async fn get_keypairs_by_round_and_scheme(
    client: HttpClient,
    round: u64,
    scheme: u32,
) -> Result<String> {
    let query = QueryRoundSchemeRequest { round, scheme };
    let res = client
        .abci_query(
            Some(
                "/azkr.tlcs.v1beta1.Query/AllKeyPairsByRoundAndScheme"
                    .parse()
                    .expect("hard coded path will always succeed"),
            ),
            query.encode_vec(),
            None,
            false,
        )
        .await?;

    if res.code.is_err() {
        return Err(anyhow!("node returned an error: {}", res.log));
    }

    let res = QueryAllKeyPairsResponse::decode(&*res.value)?;
    Ok(serde_json::to_string_pretty(&res)?)
}

pub async fn get_keypairs_by_time(client: HttpClient, time: i64) -> Result<String> {
    let query = QueryTimeRequest { time };
    let res = client
        .abci_query(
            Some(
                "/azkr.tlcs.v1beta1.Query/AllKeyPairsByTime"
                    .parse()
                    .expect("hard coded path will always succeed"),
            ),
            query.encode_vec(),
            None,
            false,
        )
        .await?;

    if res.code.is_err() {
        return Err(anyhow!("node returned an error: {}", res.log));
    }

    let res = QueryAllKeyPairsResponse::decode(&*res.value)?;
    Ok(serde_json::to_string_pretty(&res)?)
}

pub async fn get_all_loe_data(client: HttpClient) -> Result<String> {
    let res = client
        .abci_query(
            Some(
                "/azkr.tlcs.v1beta1.Query/AllLoeData"
                    .parse()
                    .expect("hard coded path will always succeed"),
            ),
            vec![],
            None,
            false,
        )
        .await?;

    if res.code.is_err() {
        return Err(anyhow!("node returned an error: {}", res.log));
    }

    let res = QueryAllLoeDataResponse::decode(&*res.value)?;
    Ok(serde_json::to_string_pretty(&res)?)
}

pub async fn get_loe_data_by_round(client: HttpClient, round: u64) -> Result<String> {
    let query = QueryRoundRequest { round };
    let res = client
        .abci_query(
            Some(
                "/azkr.tlcs.v1beta1.Query/AllLoeDataByRound"
                    .parse()
                    .expect("hard coded path will always succeed"),
            ),
            query.encode_vec(),
            None,
            false,
        )
        .await?;

    if res.code.is_err() {
        return Err(anyhow!("node returned an error: {}", res.log));
    }

    let res = QueryAllLoeDataResponse::decode(&*res.value)?;

    Ok(serde_json::to_string_pretty(&res)?)
}
