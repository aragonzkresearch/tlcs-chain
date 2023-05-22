use anyhow::{anyhow, Result};
use clap::{ArgMatches, Command};
use prost::Message;
use proto_messages::azkr::tlcs::v1beta1::QueryAllParticipantsContributionsResponse;
use tendermint_rpc::{Client, HttpClient};
use tokio::runtime::Runtime;

pub fn get_tlcs_query_command() -> Command {
    Command::new("tlcs")
        .about("Querying commands for the tlcs module")
        .subcommand(Command::new("participants").about("Query list of participants"))
        .subcommand_required(true)
}

pub fn run_tlcs_query_command(matches: &ArgMatches, node: &str) -> Result<String> {
    let client = HttpClient::new(node)?;

    match matches.subcommand() {
        Some(("participants", _sub_matches)) => Runtime::new()
            .expect("unclear why this would ever fail")
            .block_on(get_all_participants_contributions(client)),
        _ => unreachable!("exhausted list of subcommands and subcommand_required prevents `None`"),
    }
}

pub async fn get_all_participants_contributions(client: HttpClient) -> Result<String> {
    let res = client
        .abci_query(
            Some(
                "/azkr.tlcs.v1beta1.Query/AllParticipantsContributions"
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

    let res = QueryAllParticipantsContributionsResponse::decode(&*res.value)?;
    Ok(serde_json::to_string_pretty(&res)?)
}
