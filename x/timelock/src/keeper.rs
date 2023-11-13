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
        MsgMultiNewProcess,
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

use crate::LOE_GENESIS_TIME;
use crate::LOE_PERIOD;
use crate::LOE_PUBLIC_KEY;
use crate::SECURITY_PARAM;

// Key Prefixes
const CONTRIBUTION_THRESHOLD_KEY: [u8; 1] = [0];
const PARTICIPANT_DATA_KEY: [u8; 1] = [1];
const KEYPAIR_DATA_KEY: [u8; 1] = [2];
const LOE_DATA_KEY: [u8; 1] = [3];

// Temporary function to convert the scheme type number into string for the tlcs-rust code
pub fn scheme_to_string(scheme: u32) -> String {
    if scheme == 2 {
        "SECP256K1".to_string()
    } else {
        "BJJ".to_string()
    }
}

pub fn valid_scheme(scheme: u32) -> bool {
    scheme == 1 || scheme == 2
}

pub fn all_schemes_valid(schemes: Vec<u32>) -> bool {
    for scheme in schemes {
        if !valid_scheme(scheme) {
            return false;
        }
    }
    true
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

        (the_keys.count() + 1) as u32 // from usize
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

            tlcs_store.set(store_key, key_data.encode_to_vec());

            let round_data_vec = make_keyshare(
                LOE_PUBLIC_KEY.into(),
                msg.round,
                scheme_to_string(msg.scheme),
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
                    Ok(_) => info!("Successfully submitted keyshare data for {:?}", this_round),
                    Err(e) => info!("Failed to submit keyshare: {:?}", e),
                }
            });
        } else {
            return Err(AppError::InvalidRequest(
                "The keypair request is invalid".into(),
            ));
        }

        Ok(())
    }

    pub fn open_multi_new_process<T: Database>(
        &self,
        ctx: &mut TxContext<T, SK>,
        msg: &MsgMultiNewProcess,
    ) -> Result<(), AppError> {
        if msg.startround > 0
            && all_schemes_valid(msg.schemes.clone())
            && check_time(msg.pubkey_time)
        {
            info!(
                "NEW MULTI PROCESS TX: Starting Round: {:?}, Schemes: {:?}",
                msg.startround,
                msg.schemes
                    .iter()
                    .map(|&id| id.to_string() + ",")
                    .collect::<Vec<String>>()
            );

            let mut counter: u32 = 0;
            let mut this_round = msg.startround;

            while counter < msg.reqnum {
                this_round += msg.roundstep as u64;
                counter += 1;

                for this_scheme in msg.schemes.clone().into_iter() {
                    let keycount = self.open_process_count(ctx, this_round, this_scheme);

                    let mut store_key = KEYPAIR_DATA_KEY.to_vec();
                    store_key.append(&mut this_round.to_le_bytes().to_vec());
                    store_key.append(&mut this_scheme.to_le_bytes().to_vec());

                    let tlcs_store = ctx.get_mutable_kv_store(&self.store_key);
                    let this_pubkey_time = msg.pubkey_time + (counter * 6) as i64;

                    let key_data: RawMsgKeyPair = RawMsgKeyPair {
                        round: this_round,
                        scheme: this_scheme,
                        id: keycount,
                        pubkey_time: this_pubkey_time,
                        public_key: "".to_string(),
                        private_key: "".to_string(),
                    };

                    tlcs_store.set(store_key, key_data.encode_to_vec());
                }
            }
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
                scheme_to_string(msg.scheme),
                msg.data.clone(),
                SECURITY_PARAM,
            ) {
                //    if verify_participant_data(msg.round, msg.data.clone()) {
                let tlcs_store = ctx.get_mutable_kv_store(&self.store_key);
                let chain_data: RawMsgContribution = msg.to_owned().into();
                tlcs_store.set(store_key, chain_data.encode_to_vec());
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
        tlcs_store.set(prefix, key_data.encode_to_vec());

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

        if loe_signature_is_valid(msg.round, msg.signature.clone(), LOE_PUBLIC_KEY.into()) {
            info!("TX LOE Data stored: Round: {:?}", msg.round);
            tlcs_store.set(
                store_key,
                <MsgLoeData as Into<RawMsgLoeData>>::into(msg.to_owned()).encode_to_vec(),
            );
        } else {
            info!("TX LOE Data rejected: Round: {:?}", msg.round);
            return Err(AppError::InvalidRequest(format!(
                "Invalid loe data received. Round:{}",
                msg.round
            )));
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

    pub fn get_empty_keypairs<T: Database>(
        &self,
        ctx: &mut TxContext<T, SK>,
    ) -> (
        HashMap<Vec<u8>, RawMsgKeyPair>,
        HashMap<Vec<u8>, RawMsgKeyPair>,
    ) {
        let mut need_pub_key: HashMap<Vec<u8>, RawMsgKeyPair> = HashMap::new();
        let mut need_priv_key: HashMap<Vec<u8>, RawMsgKeyPair> = HashMap::new();

        let tlcs_store = ctx.get_kv_store(&self.store_key);
        let store_key = KEYPAIR_DATA_KEY.to_vec();
        let prefix_store = tlcs_store.get_immutable_prefix_store(store_key);
        let keypairs = prefix_store.range(..);

        for (index, keypair) in keypairs {
            let the_keys: RawMsgKeyPair = RawMsgKeyPair::decode::<Bytes>(keypair.into())
                .expect("invalid data in database - possible database corruption");
            if the_keys.public_key.is_empty() {
                need_pub_key.insert(index, the_keys);
            } else if the_keys.private_key.is_empty() {
                need_priv_key.insert(index, the_keys);
            }
        }

        (need_pub_key, need_priv_key)
    }

    pub fn make_public_keys<T: Database>(
        &self,
        ctx: &mut TxContext<T, SK>,
        new_key_list: HashMap<Vec<u8>, RawMsgKeyPair>,
        cur_time: i64,
        contribution_threshold: u32,
    ) {
        let mut tmp_store: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();

        for (key, mut keypair) in new_key_list {
            let mut all_participant_data: Vec<Vec<u8>> = vec![];
            let mut contrib_count: u32 = 0;

            if keypair.pubkey_time < cur_time {
                let round_all_participant_data =
                    self.get_this_round_all_participant_data(ctx, keypair.round, keypair.scheme);

                for (_, row) in round_all_participant_data {
                    let contribution: RawMsgContribution =
                        RawMsgContribution::decode::<Bytes>(row.into())
                            .expect("invalid data in database - possible database corruption");

                    all_participant_data.push(contribution.data);
                    contrib_count += 1;
                }

                if contrib_count > contribution_threshold {
                    info!("MAKE_PK: making key for round: {:?}", keypair.round);
                    keypair.public_key =
                        make_public_key(scheme_to_string(keypair.scheme), &all_participant_data);

                    tmp_store.insert(key, keypair.encode_to_vec());
                }
            }
        }

        let tlcs_store = ctx.get_mutable_kv_store(&self.store_key);
        for (mut k, v) in tmp_store {
            info!("MAKE_PK: storing new key");
            let mut prefix = KEYPAIR_DATA_KEY.to_vec();
            prefix.append(&mut k);
            tlcs_store.set(prefix, v)
        }
    }

    pub fn make_secret_keys<T: Database>(
        &self,
        ctx: &mut TxContext<T, SK>,
        new_key_list: HashMap<Vec<u8>, RawMsgKeyPair>,
    ) {
        let mut tmp_store: HashMap<Vec<u8>, Vec<u8>> = HashMap::new();
        let mut loe_signature: String;

        for (key, mut keypair) in new_key_list {
            let mut all_participant_data: Vec<Vec<u8>> = vec![];

            match self.get_this_round_loe_signature(ctx, keypair.round) {
                Some(data) => {
                    loe_signature = data;
                }
                None => continue,
            }

            let round_all_participant_data =
                self.get_this_round_all_participant_data(ctx, keypair.round, keypair.scheme);

            for (_, row) in round_all_participant_data {
                let contribution: RawMsgContribution =
                    RawMsgContribution::decode::<Bytes>(row.into())
                        .expect("invalid data in database - possible database corruption");

                all_participant_data.push(contribution.data.clone());
            }

            keypair.private_key = make_secret_key(
                scheme_to_string(keypair.scheme),
                loe_signature,
                all_participant_data,
            );

            tmp_store.insert(key, keypair.encode_to_vec());
        }

        let tlcs_store = ctx.get_mutable_kv_store(&self.store_key);
        for (mut k, v) in tmp_store {
            let mut prefix = KEYPAIR_DATA_KEY.to_vec();
            prefix.append(&mut k);
            tlcs_store.set(prefix, v)
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
        tlcs_store.set(prefix, new_threshold.encode_to_vec());

        Ok(())
    }

    pub fn get_this_round_all_participant_data<T: Database>(
        &self,
        ctx: &mut TxContext<T, SK>,
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

    pub fn get_this_round_loe_signature<T: Database>(
        &self,
        ctx: &mut TxContext<T, SK>,
        round: u64,
        //) -> Option<RawMsgLoeData> {
    ) -> Option<String> {
        let tlcs_store = ctx.get_kv_store(&self.store_key);

        let mut prefix = LOE_DATA_KEY.to_vec();
        prefix.append(&mut round.to_le_bytes().to_vec());
        let store_data = tlcs_store.get(&prefix);

        match store_data {
            Some(store_data) => {
                let loe_data = RawMsgLoeData::decode::<Bytes>(store_data.into())
                    .expect("invalid data in database - possible database corruption");
                Some(loe_data.signature)
            }
            None => None,
        }
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
            // Currently, can't get blocktime in here so the time filtering is done in the loe watcher
            if !keypair.public_key.is_empty() && keypair.private_key.is_empty() {
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

        tlcs_store.get_mutable_prefix_store(prefix)
    }
}
