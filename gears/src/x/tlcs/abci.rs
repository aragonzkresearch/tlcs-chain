use std::ops::RangeFrom;

use crate::{
    store::{MutablePrefixStore, Store},
    types::TxContext,
};
use bytes::Bytes;
use database::{PrefixDB, DB};
use prost::Message;

use super::crypto::aggregate_participant_data;

// TODO: move to keeper
const LOE_ROUNDS_ONE_HOUR: u32 = 20; // TODO: get proper value
const LAST_PROCESSED_ROUND_KEY: [u8; 1] = [0];
const PARTICIPANT_DATA_KEY: [u8; 1] = [1];
const PUBLIC_KEYS_KEY: [u8; 1] = [2];

pub fn begin_blocker<T: DB>(ctx: &mut TxContext<T>) {
    let last_processed_round = get_last_processed_round(ctx);
    let block_time = ctx.get_header().time.unix_timestamp();
    let process_up_to = process_up_to(block_time);

    for round in last_processed_round..process_up_to {
        let round_all_participant_data = get_this_round_all_participant_data(round, ctx);

        let round_data = round_all_participant_data.fold((0, vec![]), |mut acc, e| {
            acc.1.extend(e.1);
            acc.0 += acc.0;
            acc
        });

        if round_data.0 > 0 {
            let public_key = aggregate_participant_data(round_data.0, round_data.1);
            let mut public_keys_store = get_public_keys_store(ctx);
            public_keys_store.set(round.to_le_bytes().to_vec(), public_key);
        }
    }
}

// TODO: move to keeper
/// Returns the last loe round that was processed
fn get_last_processed_round<T: DB>(ctx: &mut TxContext<T>) -> u32 {
    let tlcs_store = ctx.get_mutable_kv_store(Store::Tlcs);

    let last_processed_round = tlcs_store.get(&LAST_PROCESSED_ROUND_KEY);

    match last_processed_round {
        None => 0, //initialize (initializing to zero means that round zero can never be processed!)
        Some(num) => u32::decode::<Bytes>(num.to_owned().into())
            .expect("invalid data in database - possible database corruption"),
    }
}

fn process_up_to(time: i64) -> u32 {
    latest_round_up_to(time) + LOE_ROUNDS_ONE_HOUR
}

/// Returns the latest loe round expected before the provided unix time
fn latest_round_up_to(time: i64) -> u32 {
    //TODO: implement this function
    return 200;
}

fn get_this_round_all_participant_data<'a, T: DB>(
    round: u32,
    ctx: &'a mut TxContext<T>,
) -> trees::iavl::Range<'a, RangeFrom<Vec<u8>>, PrefixDB<T>> {
    let tlcs_store = ctx.get_mutable_kv_store(Store::Tlcs);

    let mut prefix = PARTICIPANT_DATA_KEY.to_vec();

    prefix.append(&mut round.to_le_bytes().to_vec());

    tlcs_store.range(prefix..)
}

fn get_public_keys_store<'a, T: DB>(
    ctx: &'a mut TxContext<T>,
) -> MutablePrefixStore<'a, PrefixDB<T>> {
    let tlcs_store = ctx.get_mutable_kv_store(Store::Tlcs);
    tlcs_store.get_mutable_prefix_store(PUBLIC_KEYS_KEY.into())
}
