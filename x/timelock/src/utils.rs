use anyhow::{anyhow, Result};
use gears::{
    client::keys::key_store::DiskStore,
    crypto::{create_signed_transaction, SigningInfo},
};
use ibc_proto::cosmos::tx::v1beta1::TxRaw;
use ibc_proto::{
    cosmos::auth::v1beta1::QueryAccountResponse as RawQueryAccountResponse, protobuf::Protobuf,
};
use ibc_relayer::keyring::{Secp256k1KeyPair, SigningKeyPair};
use proto_messages::cosmos::{
    auth::v1beta1::{QueryAccountRequest, QueryAccountResponse},
    tx::v1beta1::{Fee, Message, TxBody},
};
use proto_types::AccAddress;
use serde::Serialize;
use std::str::FromStr;
use tendermint_informal::block::Height;
use tendermint_rpc::{endpoint::abci_query::AbciQuery, Client, HttpClient, Url};
use tokio::runtime::Runtime;

use crate::Config;

/// This method is used when sending transactions from within
/// Tx handlers (which is not supported by the SDK)
pub fn run_tx_command<Msg: Message, MessageGetter>(
    config: Config,
    message_getter: MessageGetter,
) -> Result<()>
where
    MessageGetter: FnOnce(AccAddress) -> Msg,
{
    let Config {
        node,
        home,
        from,
        chain_id,
    } = config;

    let key_store: DiskStore<Secp256k1KeyPair> = DiskStore::new(home)?;
    let key = key_store.get_key(&from)?;

    let fee_amount = None;
    let fee = Fee {
        amount: fee_amount,
        gas_limit: 100000000,
        payer: None,
        granter: "".into(),
    };

    let address = AccAddress::from_str(&key.account())?;
    let account = get_account_latest(address.clone(), node.clone())?;

    let signing_info = SigningInfo {
        key,
        sequence: account.account.get_sequence(),
        account_number: account.account.get_account_number(),
    };

    let tx_body = TxBody {
        messages: vec![message_getter(address)],
        memo: "".into(),
        timeout_height: 0,
        extension_options: vec![],
        non_critical_extension_options: vec![],
    };

    let tip = None;

    let raw_tx = create_signed_transaction(vec![signing_info], tx_body, fee, tip, chain_id);

    let client = HttpClient::new(node)?;
    Runtime::new()
        .expect("unclear why this would ever fail")
        .block_on(broadcast_tx_commit(client, raw_tx))?;

    Ok(())
}

pub async fn broadcast_tx_commit(client: HttpClient, raw_tx: TxRaw) -> Result<()> {
    let res = client
        .broadcast_tx_commit(prost::Message::encode_to_vec(&raw_tx))
        .await?;

    println!("{}", serde_json::to_string_pretty(&res)?);
    Ok(())
}

// NOTE: we're assuming here that the app has an auth module which handles this query
fn get_account_latest(address: AccAddress, node: Url) -> Result<QueryAccountResponse> {
    let query = QueryAccountRequest { address };

    run_query::<QueryAccountResponse, RawQueryAccountResponse>(
        query.encode_vec(),
        "/cosmos.auth.v1beta1.Query/Account".into(),
        node,
        None,
    )
}

/// Convenience method for running queries
pub fn run_query<
    Response: Protobuf<Raw> + std::convert::TryFrom<Raw> + Serialize,
    Raw: prost::Message + Default + std::convert::From<Response>,
>(
    query_bytes: Vec<u8>,
    path: String,
    node: Url,
    height: Option<Height>,
) -> Result<Response>
where
    <Response as TryFrom<Raw>>::Error: std::fmt::Display,
    <Response as ibc_proto::protobuf::erased::TryFrom<Raw>>::Error: std::fmt::Display,
{
    let client = HttpClient::new(node)?;

    let res = Runtime::new()
        .expect("unclear why this would ever fail")
        .block_on(run_query_async(client, query_bytes, height, path))?;

    if res.code.is_err() {
        return Err(anyhow!("node returned an error: {}", res.log));
    }

    Response::decode(&*res.value).map_err(|e| e.into())
}

async fn run_query_async(
    client: HttpClient,
    query_bytes: Vec<u8>,
    height: Option<Height>,
    path: String,
) -> Result<AbciQuery, tendermint_rpc::Error> {
    client
        .abci_query(Some(path), query_bytes, height, false)
        .await
}
