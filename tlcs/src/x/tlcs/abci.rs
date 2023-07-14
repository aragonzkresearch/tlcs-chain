use std::{collections::HashMap};
use tracing::info;

use crate::{
    error::AppError,
    store::{MutablePrefixStore, PrefixRange, Store},
    types::TxContext,
};
use bytes::Bytes;
use database::{PrefixDB, DB};
use prost::Message;

use proto_messages::azkr::tlcs::v1beta1::{
    MsgKeyPair, RawMsgContribution, RawMsgKeyPair, RawMsgLoeData,
};

use super::crypto::{aggregate_participant_data, make_secret_key};

use super::{
    //append_keypair,
    round_is_open
};

const LOE_GENESIS_TIME: u64 = 1677685200;
const LOE_PERIOD: u64 = 3;

const LAST_PROCESSED_ROUND_KEY: [u8; 1] = [0];

const PARTICIPANT_DATA_KEY: [u8; 1] = [1];
const KEYPAIR_DATA_KEY: [u8; 1] = [2];
const LOE_DATA_KEY: [u8; 1] = [3];

// TODO eliminate in the future to support multiple schemes
const TMP_SCHEME_ID: [u8; 1] = [1];

pub fn begin_blocker<T: DB>(ctx: &mut TxContext<T>) {
    //let _ = set_last_processed_round(ctx, 3852932);
    //return;

    let last_processed_round = get_last_processed_round(ctx);
    let block_time = ctx.get_header().time.unix_timestamp();
    let process_up_to = process_up_to(block_time);
    let latest_round = latest_round_up_to(block_time);

//     info!("BEGINBLOCKER:   process_to: {:?}", process_up_to);
//     info!("BEGINBLOCKER: latest_round: {:?}", latest_round);

    // TODO Later this should be added
    //for scheme in LIST_OF_SCHEMES {

    let mut new_key_list: Vec<MsgKeyPair> = Vec::new();

    //info!("BEGINBLOCKER: Start processing");

    for round in last_processed_round..process_up_to {

//        info!("BEGINBLOCKER:        round: {:?}", round);
//        info!("BEGINBLOCKER: round_is_open: {:?}", round_is_open(ctx, round));

        if round_is_open(ctx, round) {
            continue;
        }

        let mut cur_round: u64 = 0;
        let mut cur_data: Vec<u8> = Vec::new();

//        info!("BEGINBLOCKER: round data {:?}", round);
        
        // TODO Add the scheme in here
        let round_all_participant_data = get_this_round_all_participant_data(ctx, round);

        for (_, row) in round_all_participant_data {
//            info!("BEGINBLOCKER: processing round {:?}", round);
            let contribution: RawMsgContribution = RawMsgContribution::decode::<Bytes>(row.into())
                .expect("invalid data in database - possible database corruption");

            if cur_round != contribution.round && cur_round != 0 {
                info!("BEGINBLOCKER: creating round data for {:?}", round);
                let public_key = aggregate_participant_data(cur_data.clone());
                new_key_list.push(MsgKeyPair {
                    round: cur_round,
                    scheme: 1,
                    public_key: public_key,
                    private_key: Vec::new(),
                });
            }

            cur_round = contribution.round;
            cur_data.extend(contribution.data);
        }

        if cur_round != 0 {
            let public_key = aggregate_participant_data(cur_data.clone());
            new_key_list.push(MsgKeyPair {
                round: cur_round,
                scheme: 1,
                public_key: public_key,
                private_key: Vec::new(),
            });
        }

        for pair in &new_key_list {
            let mut prefix = KEYPAIR_DATA_KEY.to_vec();
            prefix.append(&mut pair.round.to_le_bytes().to_vec());
            prefix.append(&mut TMP_SCHEME_ID.to_vec());

            let tlcs_store = ctx.get_mutable_kv_store(Store::Tlcs);

            let key_data: RawMsgKeyPair = pair.to_owned().into();
            tlcs_store.set(prefix.into(), key_data.encode_to_vec());
        }
    }

    make_keys(ctx, block_time);

    // Maybe do something with the return value?
    let _ = set_last_processed_round(ctx, process_up_to);
}

fn make_keys<'a, T: DB>(ctx: &'a mut TxContext<T>, time: i64) {
    let tlcs_store = ctx.get_kv_store(Store::Tlcs);
    let store_key = KEYPAIR_DATA_KEY.to_vec();
    let keypairs = tlcs_store.get_immutable_prefix_store(store_key).range(..);

    let mut tmp_store: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();

    let latest_round = latest_round_up_to(time);

    for (_, keypair) in keypairs {
        let mut the_keys: RawMsgKeyPair = RawMsgKeyPair::decode::<Bytes>(keypair.into())
            .expect("invalid data in database - possible database corruption");

        let key_round = the_keys.round;

//        info!("BEGINBLOCKER: Secret key diff: {:?}, publen: {:?}", (key_round - latest_round), the_keys.public_key.len());
        // if secret key is blank and round < latest_round
        if the_keys.private_key.len() == 0 && key_round < latest_round && the_keys.public_key.len() > 0 {

            let all_round_data = get_this_round_all_participant_data(ctx, key_round);
            let loe_round_data = get_this_round_all_loe_data(ctx, key_round);

            let mut round_data: Vec<u8> = Vec::new();
            let mut count = 0;

            for (_, row) in all_round_data {
                let contribution: RawMsgContribution = RawMsgContribution::decode::<Bytes>(row.into())
                    .expect("invalid data in database - possible database corruption");

                if count == 0 {
                    round_data = contribution.data.clone();
                    count = 1;
                } else {
                    round_data.append(&mut contribution.data.clone());
                }
            }

            if key_round == loe_round_data.round {
                let secret_key = make_secret_key(
                    round_data,
                    key_round,
                    loe_round_data.signature,
                    the_keys.public_key.clone(),
                );

                the_keys.private_key = secret_key;

                // Store public and private keys
                let mut this_store_key = KEYPAIR_DATA_KEY.to_vec();
                this_store_key.append(&mut key_round.to_le_bytes().to_vec());
                this_store_key.append(&mut TMP_SCHEME_ID.to_vec());

                tmp_store.insert(this_store_key.into(), the_keys.encode_to_vec());
            }
        }
    }

    let tlcs_store = ctx.get_mutable_kv_store(Store::Tlcs);
    for (k, v) in tmp_store {
        tlcs_store.set(k, v)
    }
}

// TODO: move to keeper
/// Returns the last loe round that was processed
fn get_last_processed_round<T: DB>(ctx: &mut TxContext<T>) -> u64 {
    let tlcs_store = ctx.get_mutable_kv_store(Store::Tlcs);
    let last_processed_round = tlcs_store.get(&LAST_PROCESSED_ROUND_KEY);

    match last_processed_round {
        None => 0, //initialize (initializing to zero means that round zero can never be processed!)
        Some(num) => u64::decode::<Bytes>(num.to_owned().into())
            .expect("invalid data in database - possible database corruption"),
    }
}

fn set_last_processed_round<T: DB>(
    ctx: &mut TxContext<T>,
    last_round: u64,
) -> Result<(), AppError> {
    let tlcs_store = ctx.get_mutable_kv_store(Store::Tlcs);
    let prefix = LAST_PROCESSED_ROUND_KEY.to_vec();
    tlcs_store.set(prefix.into(), last_round.encode_to_vec());

    Ok(())
}

fn process_up_to(time: i64) -> u64 {
    latest_round_up_to(time) + LOE_PERIOD
}

/// Returns the latest loe round expected before the provided unix time
fn latest_round_up_to(time: i64) -> u64 {
    (time as u64 - LOE_GENESIS_TIME) / LOE_PERIOD
}

fn get_this_round_all_participant_data<'a, T: DB>(
    ctx: &'a TxContext<T>,
    round: u64,
) -> PrefixRange<'a, PrefixDB<T>> {
    let mut prefix = PARTICIPANT_DATA_KEY.to_vec();
    prefix.append(&mut round.to_le_bytes().to_vec());
    prefix.append(&mut TMP_SCHEME_ID.to_vec());

    let tlcs_store = ctx.get_kv_store(Store::Tlcs);
    tlcs_store.get_immutable_prefix_store(prefix).range(..)
}

fn get_this_round_all_loe_data<'a, T: DB>(ctx: &'a TxContext<T>, round: u64) -> RawMsgLoeData {
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
