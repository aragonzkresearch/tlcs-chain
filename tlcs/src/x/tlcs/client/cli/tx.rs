use std::{path::PathBuf, str::FromStr};
use std::str;

use anyhow::Result;
use clap::{arg, Arg, ArgAction, ArgMatches, Command};

use ibc_proto::{
    cosmos::tx::v1beta1::{
        mode_info::{Single, Sum},
        ModeInfo, SignDoc, TxBody, TxRaw,
    },
    protobuf::Protobuf,
};
use ibc_relayer::keyring::{Secp256k1KeyPair, SigningKeyPair};
use prost::Message;
use proto_messages::{
    azkr::tlcs::v1beta1::{
        MsgContribution,
        MsgLoeData,
    },
    cosmos::{
        base::v1beta1::{Coin, SendCoins},
        crypto::secp256k1::v1beta1::{PubKey, RawPubKey},
        tx::v1beta1::{AuthInfo, Fee, PublicKey, SignerInfo},
    },
};
use proto_types::AccAddress;
use tendermint_rpc::{Client, HttpClient};
use tokio::runtime::Runtime;
use crate::x::tlcs::crypto::{
    generate_participant_data,
    //verify_participant_data,
    //aggregate_participant_data,
};

use drand_core::HttpClient as DrandHttpClient;

use crate::{client::keys::key_store::DiskStore, x::auth::client::cli::query::get_account};

const LOE_URL: &str = "https://api.drand.sh/dbd506d6ef76e5f386f41c651dcb808c5bcbd75471cc4eafa3f4df7ad4e4c493/";

pub fn get_tlcs_tx_command() -> Command {
    Command::new("tlcs")
        .about("Tlcs transaction subcommands")
        .subcommand(
            Command::new("participate")
                .about("Send participant data")
                .arg(
                    Arg::new("from")
                        .required(true)
                        .value_parser(clap::value_parser!(String)),
                )
                .arg(
                    Arg::new("round")
                        .required(true)
                        .value_parser(clap::value_parser!(u64)),
                )
                .arg(
                    Arg::new("scheme")
                        .required(true)
                        .value_parser(clap::value_parser!(u32)),
                )
                /*
                .arg(
                    Arg::new("data")
                        .required(true)
                        .value_parser(clap::value_parser!(String)),
                )
                */
                .arg(
                    arg!(--fee)
                        .help(format!("Fee to pay along with transaction"))
                        .action(ArgAction::Set)
                        .value_parser(clap::value_parser!(Coin)),
                ),
        )
        .subcommand(
            Command::new("submit_loe_data")
                .about("Send loe data")
                .arg(
                    Arg::new("from")
                        .required(true)
                        .value_parser(clap::value_parser!(String)),
                )
                .arg(
                    Arg::new("round")
                        .required(true)
                        .value_parser(clap::value_parser!(u64)),
                )
                // TODO make this so that a value can be passed in and not automatically retrieved
                // here (well, below)
                /*
                .arg(
                    Arg::new("randomness")
                        .required(false)
                        .value_parser(clap::value_parser!(String)),
                )
                */
                .arg(
                    arg!(--fee)
                        .help(format!("Fee to pay along with transaction"))
                        .action(ArgAction::Set)
                        .value_parser(clap::value_parser!(Coin)),
                ),
        )
        .subcommand_required(true)
}

pub fn run_tlcs_tx_command(matches: &ArgMatches, node: &str, home: PathBuf) -> Result<()> {
    match matches.subcommand() {
        Some(("participate", sub_matches)) => {
            let from = sub_matches
                .get_one::<String>("from")
                .expect("from address argument is required preventing `None`")
                .to_owned();

            let round = sub_matches
                .get_one::<u64>("round")
                .expect("round argument is required preventing `None`")
                .to_owned();

            let scheme = sub_matches
                .get_one::<u32>("scheme")
                .expect("scheme argument is required preventing `None`")
                .to_owned();

            let fee = sub_matches.get_one::<Coin>("fee").cloned();

            let key_store: DiskStore<Secp256k1KeyPair> = DiskStore::new(home)?;

            let signing_key = key_store.get_key(&from)?;

            let client = HttpClient::new(node)?;
            let account = Runtime::new()
                .expect("unclear why this would ever fail")
                .block_on(get_account(
                    client,
                    AccAddress::from_str(&signing_key.account())?,
                ))?;

            let round_data_vec = generate_participant_data(round);

            /*
            if verify_participant_data(round, round_data_vec.clone()) {
                println!("Good data created. Len: {}", round_data_vec.len());
            } else {
                println!("Bad Data generation error!!!");
            }
            */

            let tx_raw = create_signed_participate_tx(
                AccAddress::from_str(&signing_key.account())?,
                round,
                scheme,
                round_data_vec,
                fee,
                account.account.get_sequence(),
                account.account.get_account_number(),
                signing_key,
            )?;

            let client = HttpClient::new(node)?;
            Runtime::new()
                .expect("unclear why this would ever fail")
                .block_on(broadcast_tx_commit(client, tx_raw))
        }
        Some(("submit_loe_data", sub_matches)) => {
            let from = sub_matches
                .get_one::<String>("from")
                .expect("from address argument is required preventing `None`")
                .to_owned();

            let round = sub_matches
                .get_one::<u64>("round")
                .expect("round argument is required preventing `None`")
                .to_owned();

            let fee = sub_matches.get_one::<Coin>("fee").cloned();
            let key_store: DiskStore<Secp256k1KeyPair> = DiskStore::new(home)?;

            let signing_key = key_store.get_key(&from)?;

            let client = HttpClient::new(node)?;
            let account = Runtime::new()
                .expect("unclear why this would ever fail")
                .block_on(get_account(
                    client,
                    AccAddress::from_str(&signing_key.account())?,
                ))?;

            let loe_data = Runtime::new()
                .expect("unclear why this would ever fail")
                .block_on(get_loe_data(round));

            let tx_raw = create_signed_loe_data_tx(
                round,
                loe_data.0,
                loe_data.1,
                fee,
                account.account.get_sequence(),
                account.account.get_account_number(),
                signing_key,
            )?;

            let client = HttpClient::new(node)?;
            Runtime::new()
                .expect("unclear why this would ever fail")
                .block_on(broadcast_tx_commit(client, tx_raw))
        }
        _ => unreachable!("exhausted list of subcommands and subcommand_required prevents `None`"),
    }
}

pub async fn get_loe_data(
    round: u64
) -> (Vec<u8>, Vec<u8>) {
    // Create a new client and retrieve the latest beacon. By default, it verifies its signature against the chain info.
    let client: DrandHttpClient = LOE_URL.try_into().unwrap();
    let loe = client.get(round).await.unwrap();
    // If you just want the latest use this instead
    //let latest = client.latest().await.unwrap();
    //let round = latest.round();
    //let randomness = round.randomness();
    (loe.randomness(), loe.signature())
}

pub async fn broadcast_tx_commit(client: HttpClient, raw_tx: TxRaw) -> Result<()> {
    let res = client
        .broadcast_tx_commit(raw_tx.encode_to_vec())
        .await
        .unwrap(); //TODO: remove unwrap

    println!("{}", serde_json::to_string_pretty(&res)?);
    Ok(())
}

pub fn create_signed_participate_tx(
    address: AccAddress,
    round: u64,
    scheme: u32,
    data: Vec<u8>,
    fee_amount: Option<Coin>,
    sequence: u64,
    account_number: u64,
    signing_key: Secp256k1KeyPair,
) -> Result<TxRaw> {
    let message = MsgContribution {
        address,
        round,
        scheme,
        data,
    };

    let tx_body = TxBody {
        messages: vec![message.into()],
        memo: "".into(),
        timeout_height: 0,
        extension_options: vec![],
        non_critical_extension_options: vec![],
    };

    let public_key = signing_key.public_key.serialize().to_vec();
    let public_key = RawPubKey { key: public_key };
    let public_key: PubKey = public_key
        .try_into()
        .expect("converting the secp256k1 library's public key will always succeed");

    let signer_infos = SignerInfo {
        public_key: Some(PublicKey::Secp256k1(public_key)),
        mode_info: Some(ModeInfo {
            sum: Some(Sum::Single(Single { mode: 1 })),
        }),
        sequence,
    };

    let fee_amount = fee_amount.map(|f| SendCoins::new(vec![f])).transpose()?; // can legitimately fail if coin amount is zero

    let fee = Fee {
        amount: fee_amount,
        gas_limit: 100000000,
        payer: None,
        granter: "".into(),
    };

    let auth_info = AuthInfo {
        signer_infos: vec![signer_infos],
        fee,
        tip: None,
    };

    let sign_doc = SignDoc {
        body_bytes: tx_body.encode_to_vec(),
        auth_info_bytes: auth_info.encode_vec(),
        chain_id: "test-chain".into(), //TODO: this should be passed in
        account_number,
    };

    let signature = signing_key.sign(&sign_doc.encode_to_vec()).unwrap(); //TODO: remove unwrap

    Ok(TxRaw {
        body_bytes: sign_doc.body_bytes,
        auth_info_bytes: sign_doc.auth_info_bytes,
        signatures: vec![signature],
    })
}

pub fn create_signed_loe_data_tx(
    round: u64,
    randomness: Vec<u8>,
    signature: Vec<u8>,
    fee_amount: Option<Coin>,
    sequence: u64,
    account_number: u64,
    signing_key: Secp256k1KeyPair,
) -> Result<TxRaw> {
    let message = MsgLoeData {
        round,
        randomness,
        signature,
    };

    let tx_body = TxBody {
        messages: vec![message.into()],
        memo: "".into(),
        timeout_height: 0,
        extension_options: vec![],
        non_critical_extension_options: vec![],
    };

    let public_key = signing_key.public_key.serialize().to_vec();
    let public_key = RawPubKey { key: public_key };
    let public_key: PubKey = public_key
        .try_into()
        .expect("converting the secp256k1 library's public key will always succeed");

    let signer_infos = SignerInfo {
        public_key: Some(PublicKey::Secp256k1(public_key)),
        mode_info: Some(ModeInfo {
            sum: Some(Sum::Single(Single { mode: 1 })),
        }),
        sequence,
    };

    let fee_amount = fee_amount.map(|f| SendCoins::new(vec![f])).transpose()?; // can legitimately fail if coin amount is zero

    let fee = Fee {
        amount: fee_amount,
        gas_limit: 100000000,
        payer: None,
        granter: "".into(),
    };

    let auth_info = AuthInfo {
        signer_infos: vec![signer_infos],
        fee,
        tip: None,
    };

    let sign_doc = SignDoc {
        body_bytes: tx_body.encode_to_vec(),
        auth_info_bytes: auth_info.encode_vec(),
        chain_id: "test-chain".into(), //TODO: this should be passed in
        account_number,
    };

    let signature = signing_key.sign(&sign_doc.encode_to_vec()).unwrap(); //TODO: remove unwrap

    Ok(TxRaw {
        body_bytes: sign_doc.body_bytes,
        auth_info_bytes: sign_doc.auth_info_bytes,
        signatures: vec![signature],
    })
}
