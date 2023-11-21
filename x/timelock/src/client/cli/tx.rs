use crate::keeper::scheme_to_string;
use crate::proto::tlcs::v1beta1::{MsgContribution, MsgLoeData, MsgMultiNewProcess, MsgNewProcess};
use anyhow::Result;
use clap::{Args, Subcommand};
use drand_core::HttpClient as DrandHttpClient;
use proto_types::AccAddress;
use std::process::exit;
use std::str;
use tokio::runtime::Runtime;

use crate::Message as TimelockMessage;
use tlcs_rust::chain_functions::make_keyshare;

use crate::LOE_PUBLIC_KEY;
use crate::LOE_URL;
use crate::SECURITY_PARAM;

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
        /// Key generation scheme. Currently supported schemes:
        ///      1  for BabyJubJub
        ///      2  forSecp256k1
        scheme: u32,
        /// Time that public key should be generated. Time is in unix timestamp format.
        public_key_time: i64,
    },
    /// Request new keypair for multiple round and schemes
    MultiKeypair {
        /// First LOE round number requesting.
        startround: u64,
        /// Number or keypairs to request.
        reqnum: u32,
        /// Number of rounds between requests.
        roundstep: u32,
        /// Comma seperated list of key generation schemes. Currently supported schemes:
        ///      1  for BabyJubJub
        ///      2  forSecp256k1
        #[arg(value_delimiter = ',', num_args = 1..)]
        schemes: Vec<u32>,
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
        TimelockCommands::MultiKeypair {
            startround,
            reqnum,
            roundstep,
            schemes,
            public_key_time,
        } => Ok(TimelockMessage::MultiNewProcess(MsgMultiNewProcess {
            address: from_address,
            startround,
            reqnum,
            roundstep,
            schemes,
            pubkey_time: public_key_time,
        })),
        TimelockCommands::Contribute { round, scheme, id } => {
            //let round_data_vec = generate_participant_data(round);
            let round_data_vec = make_keyshare(
                LOE_PUBLIC_KEY.into(),
                round,
                scheme_to_string(scheme),
                SECURITY_PARAM,
            );

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
        Ok(dat) => Ok(hex::encode(dat.signature())),
        Err(e) => Err(e),
    }

    // If you just want the latest use this instead
    //let latest = client.latest().await.unwrap();
    //let round = latest.round();
    //let randomness = round.randomness();
    //(loe.randomness(), loe.signature())
}
