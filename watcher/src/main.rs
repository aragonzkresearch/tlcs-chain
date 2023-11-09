use gears::utils::get_default_home_dir;
use std::fs;
use std::str::FromStr; // TODO: remove this and get rid of the config reading in this file
                       //use toml; // TODO: remove this and get rid of the config reading in this file

use error_chain::error_chain;
use std::thread;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
//use reqwest::Error;
//use reqwest::Client;
use serde::Deserialize;
//use serde_json::json;
use std::env;
use tendermint_rpc::Url;
use tokio::time::{sleep, Duration};

use timelock::LOE_GENESIS_TIME;
use timelock::LOE_PERIOD;
use timelock::LOE_URL;

error_chain! {
    foreign_links {
        EnvVar(env::VarError);
        HttpRequest(reqwest::Error);
    }
}

use timelock::{
    proto::tlcs::v1beta1::{MsgLoeData, RawMsgKeyPair},
    utils::run_tx_command,
    Config,
};

#[derive(Deserialize)]
struct Fileconf {
    tendermint_url: String,
    from_user: String,
    chain_id: String,
}

#[derive(Deserialize, Debug)]
pub struct LoeData {
    pub round: u64,
    pub randomness: String,
    pub signature: String,
}

#[derive(Deserialize, Debug)]
pub struct Pairs {
    pub keypairs: Vec<RawMsgKeyPair>,
}

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const TIMELOCK_URL: &str = "https://api.timelock.zone/tlcs/timelock/v1beta1/loe_data_needed";

#[tokio::main]
async fn main() -> Result<()> {
    // TODO: add command line arguments. Should override config file???
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        let _query = &args[1];
    }

    // This is only here until Kevin makes the config more universal
    let mut home_dir = get_default_home_dir(APP_NAME).unwrap();
    home_dir.push("config/resend.toml");

    //let contents = match fs::read_to_string("~/.tlcs/config/resend.toml") {
    let contents = match fs::read_to_string(home_dir) {
        Ok(s) => s,
        Err(_) => {
            "tendermint_url='http://localhost:26657'\nfrom_user='loesender'\nchain_id='test-chain'"
                .to_string()
        }
    };

    let file_conf: Fileconf = match toml::from_str(&contents) {
        Ok(d) => d,
        Err(_) => {
            panic!("File resend.toml is corrupt");
        }
    };
    // End of temp section

    let config = Config {
        node: Url::from_str(&file_conf.tendermint_url).unwrap(),
        home: get_default_home_dir(APP_NAME).unwrap(),
        from: file_conf.from_user,
        chain_id: tendermint_informal::chain::Id::try_from(file_conf.chain_id).unwrap(),
        delay: 0,
    };

    loop {
        // Get list of need LOE data rounds
        let response = reqwest::get(TIMELOCK_URL.to_string()).await?;
        let keypairs: Pairs = response.json().await?;
        //println!("Got keypair data. Len: {}", keypairs.keypairs.len());

        for keypair in keypairs.keypairs {
            //println!("Getting loe data for {}", keypair.round);
            if keypair.round < current_loe_round() {
                let loe_data: LoeData =
                    reqwest::get(format!("{}public/{}", LOE_URL, keypair.round))
                        .await?
                        .json()
                        .await?;

                send_transaction(config.clone(), loe_data);
                sleep(Duration::from_millis(6100)).await;
            }
        }

        // TODO: make sleep time configurable via config file
        sleep(Duration::from_millis(10000)).await;
    }

    fn send_transaction(config: Config, data: LoeData) {
        let this_round = data.round;
        let this_signature = data.signature;

        thread::spawn(move || {
            // This must be run inside a thread since it will block until it receives a response
            // which won't happen until this transaction has been processed.

            match run_tx_command(config, |addr| {
                timelock::Message::SubmitLoeData(MsgLoeData {
                    address: addr,
                    round: this_round,
                    signature: this_signature,
                })
            }) {
                Ok(_) => println!("Successfully submitted LOE data for {:?}", this_round),
                Err(e) => println!("Failed to submit LOE data: {:?}", e),
            }
        });
    }
}

fn current_loe_round() -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    ((now - LOE_GENESIS_TIME as i64) / LOE_PERIOD as i64) as u64
}
