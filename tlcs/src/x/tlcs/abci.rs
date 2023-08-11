use std::collections::HashMap;

use crate::{
    error::AppError,
    store::{MutablePrefixStore, PrefixRange, Store},
    types::TxContext,
};
use bytes::Bytes;
use database::{PrefixDB, DB};
use prost::Message;

use proto_messages::azkr::tlcs::v1beta1::{RawMsgContribution, RawMsgKeyPair, RawMsgLoeData};

use super::crypto::{aggregate_participant_data, make_secret_key};

const CONTRIBUTION_THRESHOLD_KEY: [u8; 1] = [0];

const PARTICIPANT_DATA_KEY: [u8; 1] = [1];
const KEYPAIR_DATA_KEY: [u8; 1] = [2];
const LOE_DATA_KEY: [u8; 1] = [3];

pub fn begin_blocker<T: DB>(ctx: &mut TxContext<T>) {
    //let _ = set_last_processed_round(ctx, 4183720);
    ////return;

    // TODO: Get this from the number of validators. For now we'll just set it here
    //let _ = set_contribution_threshold(ctx, 2);
    //let contribution_threshold = get_contribution_threshold(ctx);

    let contribution_threshold: u32 = 2;
    let block_time = ctx.get_header().time.unix_timestamp();

    //info!("BEGINBLOCKER:   process_to: {:?}", process_up_to);
    let (need_pub_keys, need_secret_keys) = get_empty_keypairs(ctx);

    make_public_keys(ctx, need_pub_keys, block_time, contribution_threshold);

    make_secret_keys(ctx, need_secret_keys);
}

fn get_empty_keypairs<'a, T: DB>(
    ctx: &'a mut TxContext<T>,
) -> (
    HashMap<Vec<u8>, RawMsgKeyPair>,
    HashMap<Vec<u8>, RawMsgKeyPair>,
) {
    let tlcs_store = ctx.get_kv_store(Store::Tlcs);
    let store_key = KEYPAIR_DATA_KEY.to_vec();
    let keypairs = tlcs_store.get_immutable_prefix_store(store_key).range(..);

    let mut need_pub_key: HashMap<Vec<u8>, RawMsgKeyPair> = HashMap::new();
    let mut need_priv_key: HashMap<Vec<u8>, RawMsgKeyPair> = HashMap::new();

    //let mut need_pub_key: Vec<RawMsgKeyPair> = Vec::new();
    //let mut need_priv_key: Vec<RawMsgKeyPair> = Vec::new();

    for (index, keypair) in keypairs {
        let the_keys: RawMsgKeyPair = RawMsgKeyPair::decode::<Bytes>(keypair.into())
            .expect("invalid data in database - possible database corruption");
        if the_keys.public_key.len() == 0 {
            need_pub_key.insert(index.into(), the_keys);
            //need_pub_key.push(the_keys);
        } else if the_keys.private_key.len() == 0 {
            need_priv_key.insert(index.into(), the_keys);
            //need_priv_key.push(the_keys);
        }
    }

    return (need_pub_key, need_priv_key);
}

fn make_public_keys<'a, T: DB>(
    ctx: &'a mut TxContext<T>,
    new_key_list: HashMap<Vec<u8>, RawMsgKeyPair>,
    cur_time: i64,
    contribution_threshold: u32,
) {
    let mut tmp_store: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();

    for (key, mut keypair) in new_key_list {
        let mut cur_data: Vec<u8> = Vec::new();
        let mut contrib_count: u32 = 0;

        if keypair.pubkey_time > cur_time {
            continue;
        }

        // TODO Add the scheme in here
        let round_all_participant_data =
            get_this_round_all_participant_data(ctx, keypair.round, keypair.scheme);

        for (_, row) in round_all_participant_data {
            let contribution: RawMsgContribution = RawMsgContribution::decode::<Bytes>(row.into())
                .expect("invalid data in database - possible database corruption");

            cur_data.extend(contribution.data);
            contrib_count += 1;
        }

        if contrib_count < contribution_threshold {
            continue;
        }

        let public_key = aggregate_participant_data(cur_data.clone());
        keypair.public_key = public_key;

        tmp_store.insert(key, keypair.encode_to_vec());
    }

    let tlcs_store = ctx.get_mutable_kv_store(Store::Tlcs);
    for (k, v) in tmp_store {
        tlcs_store.set(k, v)
    }
}

fn make_secret_keys<'a, T: DB>(
    ctx: &'a mut TxContext<T>,
    new_key_list: HashMap<Vec<u8>, RawMsgKeyPair>,
) {
    let mut tmp_store: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();

    for (key, mut keypair) in new_key_list {
        let mut cur_data: Vec<u8> = Vec::new();
        let loe_round_data = get_this_round_all_loe_data(ctx, keypair.round);

        if loe_round_data.signature.len() < 1 {
            continue;
        }

        let round_all_participant_data =
            get_this_round_all_participant_data(ctx, keypair.round, keypair.scheme);
        for (_, row) in round_all_participant_data {
            let contribution: RawMsgContribution = RawMsgContribution::decode::<Bytes>(row.into())
                .expect("invalid data in database - possible database corruption");

            cur_data.extend(contribution.data);
        }

        let secret_key = make_secret_key(
            cur_data,
            keypair.round,
            loe_round_data.signature,
            keypair.public_key.clone(),
        );

        keypair.private_key = secret_key;

        tmp_store.insert(key, keypair.encode_to_vec());
    }

    let tlcs_store = ctx.get_mutable_kv_store(Store::Tlcs);
    for (k, v) in tmp_store {
        tlcs_store.set(k, v)
    }
}

// TODO: move to keeper?
#[allow(dead_code)]
fn get_contribution_threshold<T: DB>(ctx: &mut TxContext<T>) -> u32 {
    let tlcs_store = ctx.get_mutable_kv_store(Store::Tlcs);
    let contribution_threshold = tlcs_store.get(&CONTRIBUTION_THRESHOLD_KEY);

    match contribution_threshold {
        None => 0, //initialize (initializing to zero means that round zero can never be processed!)
        Some(num) => u32::decode::<Bytes>(num.to_owned().into())
            .expect("invalid data in database - possible database corruption"),
    }
}

#[allow(dead_code)]
fn set_contribution_threshold<T: DB>(
    ctx: &mut TxContext<T>,
    new_threshold: u32,
) -> Result<(), AppError> {
    let tlcs_store = ctx.get_mutable_kv_store(Store::Tlcs);
    let prefix = CONTRIBUTION_THRESHOLD_KEY.to_vec();
    tlcs_store.set(prefix.into(), new_threshold.encode_to_vec());

    Ok(())
}

fn get_this_round_all_participant_data<'a, T: DB>(
    ctx: &'a mut TxContext<T>,
    round: u64,
    scheme: u32,
) -> PrefixRange<'a, PrefixDB<T>> {
    let mut prefix = PARTICIPANT_DATA_KEY.to_vec();
    prefix.append(&mut round.to_le_bytes().to_vec());
    prefix.append(&mut scheme.to_le_bytes().to_vec());

    let tlcs_store = ctx.get_kv_store(Store::Tlcs);
    tlcs_store.get_immutable_prefix_store(prefix).range(..)
}

fn get_this_round_all_loe_data<'a, T: DB>(ctx: &'a mut TxContext<T>, round: u64) -> RawMsgLoeData {
    let tlcs_store = ctx.get_kv_store(Store::Tlcs);

    let mut prefix = LOE_DATA_KEY.to_vec();
    prefix.append(&mut round.to_le_bytes().to_vec());
    //prefix.append(&mut TMP_SCHEME_ID.to_vec());
    let store_data = tlcs_store.get(&prefix);

    let loe_data = match store_data {
        Some(store_data) => RawMsgLoeData::decode::<Bytes>(store_data.into())
            .expect("invalid data in database - possible database corruption"),
        None => RawMsgLoeData {
            address: "".to_string(),
            round: 0,
            randomness: vec![],
            signature: vec![],
        },
    };

    return loe_data;
}

#[allow(dead_code)]
fn get_public_keys_store<'a, T: DB>(
    ctx: &'a mut TxContext<T>,
    round: u64,
    scheme: u32,
) -> MutablePrefixStore<'a, PrefixDB<T>> {
    let tlcs_store = ctx.get_mutable_kv_store(Store::Tlcs);

    let mut prefix = KEYPAIR_DATA_KEY.to_vec();
    prefix.append(&mut round.to_le_bytes().to_vec());
    prefix.append(&mut scheme.to_le_bytes().to_vec());

    //tlcs_store.get_mutable_prefix_store(KEYPAIR_DATA_KEY.into())
    tlcs_store.get_mutable_prefix_store(prefix.into())
}
