use crate::proto::tlcs::v1beta1::{MsgContribution, MsgLoeData, MsgNewProcess};
use anyhow::Result;
use clap::{Args, Subcommand};
use drand_core::HttpClient as DrandHttpClient;
use proto_types::AccAddress;
use std::process::exit;
use std::str;
use tokio::runtime::Runtime;

use crate::Message as TimelockMessage;
use tlcs_rust::chain_functions::make_keyshare;

const LOE_PUBLIC_KEY: &str = "a0b862a7527fee3a731bcb59280ab6abd62d5c0b6ea03dc4ddf6612fdfc9d01f01c31542541771903475eb1ec6615f8d0df0b8b6dce385811d6dcf8cbefb8759e5e616a3dfd054c928940766d9a5b9db91e3b697e5d70a975181e007f87fca5e";
const SECURITY_PARAM: usize = 10;
const SCHEME: &str = "BJJ";
const LOE_URL: &str =
    "https://api.drand.sh/dbd506d6ef76e5f386f41c651dcb808c5bcbd75471cc4eafa3f4df7ad4e4c493/";

#[derive(Args, Debug)]
pub struct Cli {
    #[command(subcommand)]
    command: TimelockCommands,
}

#[derive(Subcommand, Debug)]
pub enum TimelockCommands {
    /// Request new keypair for given round and scheme
    Keypair {
        /// LOE round number.
        round: u64,
        /// Key generation scheme. Currently must be 1.
        scheme: u32,
        /// Time that public key should be generated. Time is in unix timestamp format.
        public_key_time: i64,
    },
    /// Send contribution data for given round and scheme
    Contribute {
        /// LOE round number.
        round: u64,
        /// Key generation scheme. Currently must be 1.
        scheme: u32,
        /// Contribution ID.
        id: u32,
    },
    /// Submit LOE data
    Submit {
        /// LOE round number.
        round: u64,
        // Signature.
        //signature: Option<String>,
    },
}

pub fn run_timelock_tx_command(args: Cli, from_address: AccAddress) -> Result<TimelockMessage> {
    match args.command {
        TimelockCommands::Keypair {
            round,
            scheme,
            public_key_time,
        } => Ok(TimelockMessage::NewProcess(MsgNewProcess {
            address: from_address,
            round,
            scheme,
            pubkey_time: public_key_time,
        })),
        TimelockCommands::Contribute { round, scheme, id } => {
            //let round_data_vec = generate_participant_data(round);
            let round_data_vec =
                make_keyshare(LOE_PUBLIC_KEY.into(), round, SCHEME.into(), SECURITY_PARAM);

            Ok(TimelockMessage::Participate(MsgContribution {
                address: from_address,
                round,
                scheme,
                id,
                data: round_data_vec,
            }))
        }
        TimelockCommands::Submit { round } => {
            // TODO make this so that signature can be passed in and not automatically retrieved

            println!("Retrieving LOE Data from API");
            let loe_data = match Runtime::new()
                .expect("unclear why this would ever fail")
                .block_on(get_loe_data(round))
            {
                Ok(dat) => dat,
                Err(e) => {
                    println!("Error Retrieving LOE Data. Try again later.\nError: {}", e);
                    exit(1);
                }
            };

            println!("Round: {:?}", round);
            println!("Sig: {:?}", loe_data);

            Ok(TimelockMessage::SubmitLoeData(MsgLoeData {
                address: from_address,
                round,
                signature: loe_data,
            }))
        }
    }
}

pub async fn get_loe_data(round: u64) -> Result<String> {
    // Create a new client and retrieve the latest beacon. By default, it verifies its signature against the chain info.
    let client: DrandHttpClient = LOE_URL.try_into().unwrap();
    match client.get(round).await {
        Ok(dat) => {
            return Ok(hex::encode(dat.signature()));
        }
        Err(e) => return Err(e),
    };

    // If you just want the latest use this instead
    //let latest = client.latest().await.unwrap();
    //let round = latest.round();
    //let randomness = round.randomness();
    //(loe.randomness(), loe.signature())
}
