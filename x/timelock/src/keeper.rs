use std::{collections::HashMap, thread};

use bytes::Bytes;
use database::{Database, PrefixDB};
use gears::{
    error::AppError,
    types::context::{Context, QueryContext, TxContext},
};
use prost::Message;
use store::{MutablePrefixStore, StoreKey};
use tracing::info;
// Include to run benchmark and uncomment benchmark in test
//use std::time::Instant;
use tlcs_rust::chain_functions::{
    loe_signature_is_valid, make_keyshare, make_public_key, make_secret_key, verify_keyshare,
};

use crate::{
    proto::tlcs::v1beta1::{
        MsgContribution,
        MsgKeyPair,
        MsgLoeData,
        MsgNewProcess,
        QueryAllContributionsResponse,
        QueryAllKeyPairsResponse,
        QueryAllLoeDataResponse,
        RawMsgContribution,
        RawMsgKeyPair,
        RawMsgLoeData,
        //RawMsgNewProcess,
    },
    utils::run_tx_command,
    Config,
};

use chrono::Utc;

// Key Prefixes
const CONTRIBUTION_THRESHOLD_KEY: [u8; 1] = [0];
const PARTICIPANT_DATA_KEY: [u8; 1] = [1];
const KEYPAIR_DATA_KEY: [u8; 1] = [2];
const LOE_DATA_KEY: [u8; 1] = [3];

const LOE_PUBLIC_KEY: &str = "a0b862a7527fee3a731bcb59280ab6abd62d5c0b6ea03dc4ddf6612fdfc9d01f01c31542541771903475eb1ec6615f8d0df0b8b6dce385811d6dcf8cbefb8759e5e616a3dfd054c928940766d9a5b9db91e3b697e5d70a975181e007f87fca5e";
const SECURITY_PARAM: usize = 10;
const SCHEME: &str = "BJJ";

const LOE_GENESIS_TIME: u32 = 1677685200;
const LOE_PERIOD: u32 = 3;

pub fn valid_scheme(scheme: u32) -> bool {
    scheme == 1
}

pub fn check_time(time: i64) -> bool {
    let now = Utc::now();
    time > now.timestamp()
}

#[derive(Debug, Clone)]
pub struct Keeper<SK: StoreKey> {
    store_key: SK,
}

impl<SK: StoreKey> Keeper<SK> {
    pub fn new(store_key: SK) -> Self {
        Keeper { store_key }
    }

    pub fn open_process_count<T: Database>(
        &self,
        ctx: &mut TxContext<T, SK>,
        round: u64,
        scheme: u32,
    ) -> u32 {
        let tlcs_store = ctx.get_kv_store(&self.store_key);

        // Make store search key
        let mut store_key = KEYPAIR_DATA_KEY.to_vec();
        store_key.append(&mut round.to_le_bytes().to_vec());
        store_key.append(&mut scheme.to_le_bytes().to_vec());

        let prefix_store = tlcs_store.get_immutable_prefix_store(store_key);
        let the_keys = prefix_store.range(..);
        let keycount = the_keys.count() as u32; // usize

        return keycount;
    }

    pub fn open_new_process<T: Database>(
        &self,
        ctx: &mut TxContext<T, SK>,
        config: Config,
        msg: &MsgNewProcess,
    ) -> Result<(), AppError> {
        if msg.round > 0 && valid_scheme(msg.scheme) && check_time(msg.pubkey_time) {
            info!(
                "NEW PROCESS TX: Round: {:?}, Scheme: {:?}",
                msg.round, msg.scheme
            );

            let keycount = self.open_process_count(ctx, msg.round, msg.scheme);

            let mut store_key = KEYPAIR_DATA_KEY.to_vec();
            store_key.append(&mut msg.round.to_le_bytes().to_vec());
            store_key.append(&mut msg.scheme.to_le_bytes().to_vec());

            let tlcs_store = ctx.get_mutable_kv_store(&self.store_key);

            let key_data: RawMsgKeyPair = RawMsgKeyPair {
                round: msg.round,
                scheme: msg.scheme,
                id: keycount,
                pubkey_time: msg.pubkey_time,
                public_key: "".to_string(),
                private_key: "".to_string(),
            };

            tlcs_store.set(store_key.into(), key_data.encode_to_vec());

            let round_data_vec = make_keyshare(
                LOE_PUBLIC_KEY.into(),
                msg.round,
                SCHEME.into(),
                SECURITY_PARAM,
            );
            let this_round = msg.round;
            let this_scheme = msg.scheme;

            thread::spawn(move || {
                // This must be run inside a thread since it will block until it receives a response
                // which won't happen until this transaction has been processed.

                match run_tx_command(config, |addr| {
                    crate::Message::Participate(MsgContribution {
                        address: addr,
                        round: this_round,
                        scheme: this_scheme,
                        id: keycount,
                        data: round_data_vec,
                    })
                }) {
                    Ok(_) => info!(
                        "Successfully submitted Contribution data for {:?}",
                        this_round
                    ),
                    Err(e) => info!("Failed to submit Contribution: {:?}", e),
                }
            });
        } else {
            return Err(AppError::InvalidRequest(
                "The keypair request is invalid".into(),
            ));
        }

        Ok(())
    }

    pub fn append_contribution<T: Database>(
        &self,
        ctx: &mut TxContext<T, SK>,
        msg: &MsgContribution,
    ) -> Result<(), AppError> {
        // Make store key
        let mut store_key = PARTICIPANT_DATA_KEY.to_vec();
        store_key.append(&mut msg.round.to_le_bytes().to_vec());
        store_key.append(&mut msg.scheme.to_le_bytes().to_vec());

        let addr: Vec<u8> = msg.address.clone().into();
        store_key.append(&mut addr.to_vec());

        if !valid_scheme(msg.scheme) {
            return Err(AppError::InvalidRequest("Invalid scheme.".into()));
        }

        info!(
            "CONTRIB TX: new data. Round: {:?}, Scheme: {:?}",
            msg.round, msg.scheme
        );

        let keycount = self.open_process_count(ctx, msg.round, msg.scheme);

        if msg.id <= (keycount - 1) {
            if verify_keyshare(
                LOE_PUBLIC_KEY.into(),
                msg.round,
                SCHEME.into(),
                msg.data.clone(),
                SECURITY_PARAM,
            ) {
                //    if verify_participant_data(msg.round, msg.data.clone()) {
                let tlcs_store = ctx.get_mutable_kv_store(&self.store_key);
                let chain_data: RawMsgContribution = msg.to_owned().into();
                tlcs_store.set(store_key.into(), chain_data.encode_to_vec());
            } else {
                return Err(AppError::InvalidRequest(
                    "The contribution data is invalid for the given round".into(),
                ));
            }
        } else {
            return Err(AppError::InvalidRequest(
                "Can't contribute data without existing keypair request.".into(),
            ));
        }

        Ok(())
    }

    pub fn query_all_contributions<T: Database>(
        &self,
        ctx: &QueryContext<T, SK>,
    ) -> QueryAllContributionsResponse {
        let store_key = PARTICIPANT_DATA_KEY.to_vec();

        let tlcs_store = ctx.get_kv_store(&self.store_key);
        let prefix_store = tlcs_store.get_immutable_prefix_store(store_key);
        let all_raw_data = prefix_store.range(..);

        let mut contributions = vec![];

        for (_, row) in all_raw_data {
            let contribution: RawMsgContribution = RawMsgContribution::decode::<Bytes>(row.into())
                .expect("invalid data in database - possible database corruption");
            contributions.push(contribution);
        }
        QueryAllContributionsResponse { contributions }
    }

    pub fn query_contributions_by_round<T: Database>(
        &self,
        ctx: &QueryContext<T, SK>,
        round: u64,
    ) -> QueryAllContributionsResponse {
        let tlcs_store = ctx.get_kv_store(&self.store_key);

        let mut store_key = PARTICIPANT_DATA_KEY.to_vec();
        store_key.append(&mut round.to_le_bytes().to_vec());

        let prefix_store = tlcs_store.get_immutable_prefix_store(store_key);
        let all_raw_data = prefix_store.range(..);

        let mut contributions = vec![];

        for (_, row) in all_raw_data {
            let contribution: RawMsgContribution = RawMsgContribution::decode::<Bytes>(row.into())
                .expect("invalid data in database - possible database corruption");
            contributions.push(contribution);
        }
        QueryAllContributionsResponse { contributions }
    }

    pub fn query_contributions_by_round_and_scheme<T: Database>(
        &self,
        ctx: &QueryContext<T, SK>,
        round: u64,
        scheme: u32,
    ) -> QueryAllContributionsResponse {
        let tlcs_store = ctx.get_kv_store(&self.store_key);

        let mut store_key = PARTICIPANT_DATA_KEY.to_vec();
        store_key.append(&mut round.to_le_bytes().to_vec());
        store_key.append(&mut scheme.to_le_bytes().to_vec());

        let prefix_store = tlcs_store.get_immutable_prefix_store(store_key);
        let all_raw_data = prefix_store.range(..);

        let mut contributions = vec![];

        for (_, row) in all_raw_data {
            let contribution: RawMsgContribution = RawMsgContribution::decode::<Bytes>(row.into())
                .expect("invalid data in database - possible database corruption");
            contributions.push(contribution);
        }
        QueryAllContributionsResponse { contributions }
    }

    // Keypair section

    // TODO maybe use this from the endblocker so all of the save/query functions are in the keeper
    #[allow(dead_code)]
    pub fn append_keypair<T: Database>(
        &self,
        ctx: &mut TxContext<T, SK>,
        msg: &MsgKeyPair,
    ) -> Result<(), AppError> {
        let mut prefix = KEYPAIR_DATA_KEY.to_vec();
        prefix.append(&mut msg.round.to_le_bytes().to_vec());
        prefix.append(&mut msg.scheme.to_le_bytes().to_vec());

        if !valid_scheme(msg.scheme) {
            return Err(AppError::InvalidRequest("Invalid scheme.".into()));
        }

        let tlcs_store = ctx.get_mutable_kv_store(&self.store_key);

        let key_data: RawMsgKeyPair = msg.to_owned().into();
        tlcs_store.set(prefix.into(), key_data.encode_to_vec());

        Ok(())
    }

    pub fn query_all_keypairs<T: Database>(
        &self,
        ctx: &QueryContext<T, SK>,
    ) -> QueryAllKeyPairsResponse {
        let tlcs_store = ctx.get_kv_store(&self.store_key);
        let store_key = KEYPAIR_DATA_KEY.to_vec();

        let prefix_store = tlcs_store.get_immutable_prefix_store(store_key);
        let all_raw_data = prefix_store.range(..);

        let mut keypairs = vec![];

        for (_, row) in all_raw_data {
            let keypair: RawMsgKeyPair = RawMsgKeyPair::decode::<Bytes>(row.into())
                .expect("invalid data in database - possible database corruption");
            keypairs.push(keypair);
        }
        QueryAllKeyPairsResponse { keypairs }
    }

    pub fn query_keypairs_by_round<T: Database>(
        &self,
        ctx: &QueryContext<T, SK>,
        round: u64,
    ) -> QueryAllKeyPairsResponse {
        let tlcs_store = ctx.get_kv_store(&self.store_key);
        let mut store_key = KEYPAIR_DATA_KEY.to_vec();
        store_key.append(&mut round.to_le_bytes().to_vec());

        let prefix_store = tlcs_store.get_immutable_prefix_store(store_key);
        let all_raw_data = prefix_store.range(..);

        let mut keypairs = vec![];

        for (_, row) in all_raw_data {
            let keypair: RawMsgKeyPair = RawMsgKeyPair::decode::<Bytes>(row.into())
                .expect("invalid data in database - possible database corruption");
            keypairs.push(keypair);
        }
        QueryAllKeyPairsResponse { keypairs }
    }

    pub fn query_keypairs_by_time<T: Database>(
        &self,
        ctx: &QueryContext<T, SK>,
        time: i64,
    ) -> QueryAllKeyPairsResponse {
        let tlcs_store = ctx.get_kv_store(&self.store_key);
        let mut store_key = KEYPAIR_DATA_KEY.to_vec();

        let latest_round = (time as u32 - LOE_GENESIS_TIME) / LOE_PERIOD;

        store_key.append(&mut latest_round.to_le_bytes().to_vec());
        let prefix_store = tlcs_store.get_immutable_prefix_store(store_key);
        let all_raw_data = prefix_store.range(..);

        let mut keypairs = vec![];

        for (_, row) in all_raw_data {
            let keypair: RawMsgKeyPair = RawMsgKeyPair::decode::<Bytes>(row.into())
                .expect("invalid data in database - possible database corruption");
            keypairs.push(keypair);
        }
        QueryAllKeyPairsResponse { keypairs }
    }

    pub fn query_keypairs_by_round_and_scheme<T: Database>(
        &self,
        ctx: &QueryContext<T, SK>,
        round: u64,
        scheme: u32,
    ) -> QueryAllKeyPairsResponse {
        let tlcs_store = ctx.get_kv_store(&self.store_key);

        let mut store_key = KEYPAIR_DATA_KEY.to_vec();
        store_key.append(&mut round.to_le_bytes().to_vec());
        store_key.append(&mut scheme.to_le_bytes().to_vec());

        let prefix_store = tlcs_store.get_immutable_prefix_store(store_key);
        let all_raw_data = prefix_store.range(..);

        let mut keypairs = vec![];

        for (_, row) in all_raw_data {
            let keypair: RawMsgKeyPair = RawMsgKeyPair::decode::<Bytes>(row.into())
                .expect("invalid data in database - possible database corruption");
            keypairs.push(keypair);
        }

        QueryAllKeyPairsResponse { keypairs }
    }

    pub fn append_loe_data<T: Database>(
        &self,
        ctx: &mut Context<T, SK>,
        msg: &MsgLoeData,
    ) -> Result<(), AppError> {
        let tlcs_store = ctx.get_mutable_kv_store(&self.store_key);

        let mut store_key = LOE_DATA_KEY.to_vec();
        store_key.append(&mut msg.round.to_le_bytes().to_vec());

        //if loe_signature_is_valid(msg.round, msg.signature.clone()) {
        if loe_signature_is_valid(msg.round, msg.signature.clone(), LOE_PUBLIC_KEY.into()) {
            //if loe_signature_is_valid(msg.round, msg.signature.clone()) {
            tlcs_store.set(
                store_key.into(),
                <MsgLoeData as Into<RawMsgLoeData>>::into(msg.to_owned()).encode_to_vec(),
            );
        } else {
            return Err(AppError::InvalidRequest(
                "the loe data is invalid for the given round".into(),
            ));
        }

        Ok(())
    }

    pub fn query_all_loe_data<T: Database>(
        &self,
        ctx: &QueryContext<T, SK>,
    ) -> QueryAllLoeDataResponse {
        let tlcs_store = ctx.get_kv_store(&self.store_key);
        let store_key = LOE_DATA_KEY.to_vec();
        let prefix_store = tlcs_store.get_immutable_prefix_store(store_key);
        let all_raw_data = prefix_store.range(..);

        let mut randomnesses = vec![];

        for (_, row) in all_raw_data {
            let loe_data: RawMsgLoeData = RawMsgLoeData::decode::<Bytes>(row.into())
                .expect("invalid data in database - possible database corruption");
            randomnesses.push(loe_data);
        }

        QueryAllLoeDataResponse { randomnesses }
    }

    pub fn query_loe_data_by_round<T: Database>(
        &self,
        ctx: &QueryContext<T, SK>,
        round: u64,
    ) -> QueryAllLoeDataResponse {
        let tlcs_store = ctx.get_kv_store(&self.store_key);
        let mut store_key = KEYPAIR_DATA_KEY.to_vec();
        store_key.append(&mut round.to_le_bytes().to_vec());
        let prefix_store = tlcs_store.get_immutable_prefix_store(store_key);
        let all_raw_data = prefix_store.range(..);

        let mut randomnesses = vec![];

        for (_, row) in all_raw_data {
            let rand: RawMsgLoeData = RawMsgLoeData::decode::<Bytes>(row.into())
                .expect("invalid data in database - possible database corruption");
            randomnesses.push(rand);
        }

        QueryAllLoeDataResponse { randomnesses }
    }

    pub fn get_empty_keypairs<'a, T: Database>(
        &self,
        ctx: &'a mut TxContext<T, SK>,
    ) -> (
        HashMap<Vec<u8>, RawMsgKeyPair>,
        HashMap<Vec<u8>, RawMsgKeyPair>,
    ) {
        let tlcs_store = ctx.get_kv_store(&self.store_key);
        let store_key = KEYPAIR_DATA_KEY.to_vec();
        let prefix_store = tlcs_store.get_immutable_prefix_store(store_key);
        let keypairs = prefix_store.range(..);

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

    // self.keeper.make_public_keys(ctx, need_pub_keys, block_time, contribution_threshold);
    pub fn make_public_keys<'a, T: Database>(
        &self,
        ctx: &'a mut TxContext<T, SK>,
        new_key_list: HashMap<Vec<u8>, RawMsgKeyPair>,
        cur_time: i64,
        contribution_threshold: u32,
    ) {
        let mut tmp_store: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();

        for (key, mut keypair) in new_key_list {
            let mut all_participant_data: Vec<Vec<u8>> = vec![];
            let mut contrib_count: u32 = 0;

            info!(
                "Make PK: key time: {:?}, cur time: {:?}",
                keypair.pubkey_time, cur_time
            );
            if keypair.pubkey_time > cur_time {
                continue;
            }

            // TODO Add the scheme in here
            let round_all_participant_data =
                self.get_this_round_all_participant_data(ctx, keypair.round, keypair.scheme);

            for (_, row) in round_all_participant_data {
                let contribution: RawMsgContribution =
                    RawMsgContribution::decode::<Bytes>(row.into())
                        .expect("invalid data in database - possible database corruption");

                //cur_data.extend(contribution.data);
                all_participant_data.push(contribution.data);
                contrib_count += 1;
            }

            if contrib_count < contribution_threshold {
                continue;
            }

            let public_key = make_public_key(LOE_PUBLIC_KEY.into(), &all_participant_data);
            keypair.public_key = hex::encode(&public_key);

            tmp_store.insert(key, keypair.encode_to_vec());
        }

        let tlcs_store = ctx.get_mutable_kv_store(&self.store_key);
        for (k, v) in tmp_store {
            tlcs_store.set(k, v)
        }
    }

    pub fn make_secret_keys<'a, T: Database>(
        &self,
        ctx: &'a mut TxContext<T, SK>,
        new_key_list: HashMap<Vec<u8>, RawMsgKeyPair>,
    ) {
        let mut tmp_store: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
        let mut loe_signature: String;

        for (key, mut keypair) in new_key_list {
            let mut all_participant_data: Vec<Vec<u8>> = vec![];
            match self.get_this_round_all_loe_data(ctx, keypair.round) {
                Some(data) => {
                    loe_signature = hex::encode(data.signature);
                }
                None => continue,
            }

            let round_all_participant_data =
                self.get_this_round_all_participant_data(ctx, keypair.round, keypair.scheme);

            for (_, row) in round_all_participant_data {
                let contribution: RawMsgContribution =
                    RawMsgContribution::decode::<Bytes>(row.into())
                        .expect("invalid data in database - possible database corruption");

                all_participant_data.push(contribution.data);
            }

            let secret_key = make_secret_key(
                keypair.round,
                SCHEME.to_string(),
                loe_signature,
                all_participant_data,
            );

            keypair.private_key = hex::encode(secret_key);

            tmp_store.insert(key, keypair.encode_to_vec());
        }

        let tlcs_store = ctx.get_mutable_kv_store(&self.store_key);
        for (k, v) in tmp_store {
            tlcs_store.set(k, v)
        }
    }

    #[allow(dead_code)]
    pub fn get_contribution_threshold<T: Database>(&self, ctx: &mut TxContext<T, SK>) -> u32 {
        let tlcs_store = ctx.get_mutable_kv_store(&self.store_key);
        let contribution_threshold = tlcs_store.get(&CONTRIBUTION_THRESHOLD_KEY);

        match contribution_threshold {
            None => 0, //initialize (initializing to zero means that round zero can never be processed!)
            Some(num) => u32::decode::<Bytes>(num.to_owned().into())
                .expect("invalid data in database - possible database corruption"),
        }
    }

    #[allow(dead_code)]
    pub fn set_contribution_threshold<T: Database>(
        &self,
        ctx: &mut TxContext<T, SK>,
        new_threshold: u32,
    ) -> Result<(), AppError> {
        let tlcs_store = ctx.get_mutable_kv_store(&self.store_key);
        let prefix = CONTRIBUTION_THRESHOLD_KEY.to_vec();
        tlcs_store.set(prefix.into(), new_threshold.encode_to_vec());

        Ok(())
    }

    pub fn get_this_round_all_participant_data<'a, T: Database>(
        &self,
        ctx: &'a mut TxContext<T, SK>,
        round: u64,
        scheme: u32,
    ) -> Vec<(Vec<u8>, Vec<u8>)> {
        let mut prefix = PARTICIPANT_DATA_KEY.to_vec();
        prefix.append(&mut round.to_le_bytes().to_vec());
        prefix.append(&mut scheme.to_le_bytes().to_vec());

        let tlcs_store = ctx.get_kv_store(&self.store_key);
        tlcs_store
            .get_immutable_prefix_store(prefix)
            .range(..)
            .collect()
    }

    pub fn get_this_round_all_loe_data<'a, T: Database>(
        &self,
        ctx: &'a mut TxContext<T, SK>,
        round: u64,
    ) -> Option<RawMsgLoeData> {
        let tlcs_store = ctx.get_kv_store(&self.store_key);

        let mut prefix = LOE_DATA_KEY.to_vec();
        prefix.append(&mut round.to_le_bytes().to_vec());
        let store_data = tlcs_store.get(&prefix);

        match store_data {
            Some(store_data) => {
                return Some(
                    RawMsgLoeData::decode::<Bytes>(store_data.into())
                        .expect("invalid data in database - possible database corruption"),
                )
            }
            None => return None,
        };
    }

    pub fn query_loe_data_needed<T: Database>(
        &self,
        ctx: &QueryContext<T, SK>,
    ) -> QueryAllKeyPairsResponse {
        let tlcs_store = ctx.get_kv_store(&self.store_key);
        let store_key = KEYPAIR_DATA_KEY.to_vec();

        let prefix_store = tlcs_store.get_immutable_prefix_store(store_key);
        let all_raw_data = prefix_store.range(..);

        let mut keypairs = vec![];

        for (_, row) in all_raw_data {
            let keypair: RawMsgKeyPair = RawMsgKeyPair::decode::<Bytes>(row.into())
                .expect("invalid data in database - possible database corruption");
            //TODO: Possibly also filter by blocktime. It would be better but for now we'll just get records with empty private keys
            if keypair.public_key != "" && keypair.private_key == "" {
                keypairs.push(keypair);
            }
        }
        QueryAllKeyPairsResponse { keypairs }
    }

    #[allow(dead_code)]
    pub fn get_public_keys_store<'a, T: Database>(
        &self,
        ctx: &'a mut TxContext<T, SK>,
        round: u64,
        scheme: u32,
    ) -> MutablePrefixStore<'a, PrefixDB<T>> {
        let tlcs_store = ctx.get_mutable_kv_store(&self.store_key);

        let mut prefix = KEYPAIR_DATA_KEY.to_vec();
        prefix.append(&mut round.to_le_bytes().to_vec());
        prefix.append(&mut scheme.to_le_bytes().to_vec());

        //tlcs_store.get_mutable_prefix_store(KEYPAIR_DATA_KEY.into())
        tlcs_store.get_mutable_prefix_store(prefix.into())
    }
}

#[test]
fn test_round_signature() {
    //let signature: String = "9544ddce2fdbe8688d6f5b4f98eed5d63eee3902e7e162050ac0f45905a55657714880adabe3c3096b92767d886567d0".to_string();
    //let round: u32 = 1;
    //let randomness: Vec<u8> = hex::decode("f282310f131ed63e0342cd7e47f9e4317b20fb6f652b03ce81378cf825227212").unwrap();
    let signature: &str = "86f91b1eec7b22ecce1385ec1cc4861f43507fa897cad686e44a87986a7ce18a94fa7128d6f76d6b950bb4e559472539";
    let round: u64 = 3276594;
    /* Benchmark
    let before = Instant::now();
    for _i in 0..1000 {
        //_ = loe_signature_is_valid(round, randomness.clone(), signature.clone());
        _ = loe_signature_is_valid(round, signature.clone());
        // some code
    }
    println!("Elapsed time(10000): {:.2?}", before.elapsed());
    */
    //assert!(loe_signature_is_valid(round, randomness, signature));
    assert!(loe_signature_is_valid(
        round,
        signature.into(),
        LOE_PUBLIC_KEY.into()
    ));
}
