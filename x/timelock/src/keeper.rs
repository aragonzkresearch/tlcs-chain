use bytes::Bytes;
use database::Database;
use gears::{
    error::AppError,
    types::context::{Context, QueryContext, TxContext},
};
use prost::Message;
use proto_types::AccAddress;
use store::StoreKey;
use tracing::info;
// Include to run benchmark and uncomment benchmark in test
//use std::time::Instant;

use crate::proto::tlcs::v1beta1::{
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
};

use crate::crypto::verify_participant_data;

// For LOE data verification
use drand_verify::{verify, G2Pubkey, Pubkey};
use hex_literal::hex;

// Key Prefixes
const PARTICIPANT_DATA_KEY: [u8; 1] = [1];
const KEYPAIR_DATA_KEY: [u8; 1] = [2];
const LOE_DATA_KEY: [u8; 1] = [3];

const TMP_SCHEME_ID: [u8; 1] = [1];

const LOE_PUBLIC_KEY: [u8; 96] = hex!("a0b862a7527fee3a731bcb59280ab6abd62d5c0b6ea03dc4ddf6612fdfc9d01f01c31542541771903475eb1ec6615f8d0df0b8b6dce385811d6dcf8cbefb8759e5e616a3dfd054c928940766d9a5b9db91e3b697e5d70a975181e007f87fca5e");
const LOE_GENESIS_TIME: u32 = 1677685200;
const LOE_PERIOD: u32 = 3;

pub fn valid_scheme(scheme: u32) -> bool {
    scheme == 1
}

#[derive(Debug, Clone)]
pub struct Keeper<SK: StoreKey> {
    store_key: SK,
}

impl<SK: StoreKey> Keeper<SK> {
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
        msg: &MsgNewProcess,
    ) -> Result<(), AppError> {
        if !valid_scheme(msg.scheme) {
            return Err(AppError::InvalidRequest("Invalid scheme.".into()));
        }

        info!(
            "PROCESS TX: new process request. Round: {:?}, Scheme: {:?}",
            msg.round, msg.scheme
        );

        let keycount = self.open_process_count(ctx, msg.round, msg.scheme);

        if verify_participant_data(msg.round, msg.data.clone()) {
            let tlcs_store = ctx.get_mutable_kv_store(&self.store_key);

            // Save Participant data
            let mut prefix = PARTICIPANT_DATA_KEY.to_vec();
            prefix.append(&mut msg.round.to_le_bytes().to_vec());
            prefix.append(&mut msg.scheme.to_le_bytes().to_vec());

            let addr: Vec<u8> = msg.address.clone().into();
            prefix.append(&mut addr.to_vec());

            let address = AccAddress::from_bech32(&msg.address.to_string())
                .map_err(|e| AppError::InvalidRequest(e.to_string()))?;
            //.map_err(|e| Error::DecodeAddress(e.to_string()))?;

            let contrib_data: RawMsgContribution = RawMsgContribution {
                address: address.to_string(),
                round: msg.round,
                scheme: msg.scheme,
                id: keycount,
                data: msg.data.clone(),
            };
            tlcs_store.set(prefix.into(), contrib_data.encode_to_vec());

            // Create empty keypair
            let mut prefix = KEYPAIR_DATA_KEY.to_vec();
            prefix.append(&mut msg.round.to_le_bytes().to_vec());
            prefix.append(&mut TMP_SCHEME_ID.to_vec());

            let key_data: RawMsgKeyPair = RawMsgKeyPair {
                round: msg.round,
                scheme: msg.scheme,
                id: keycount,
                pubkey_time: msg.pubkey_time,
                public_key: "".to_string(),
                private_key: "".to_string(),
            };
            tlcs_store.set(prefix.into(), key_data.encode_to_vec());
        } else {
            return Err(AppError::InvalidRequest(
                "The contribution data is invalid for the given round".into(),
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

        if keycount < 1 {
            if keycount < msg.id {
                return Err(AppError::InvalidRequest(
                    "The round is no longer open for contributions".into(),
                ));
            }

            if verify_participant_data(msg.round, msg.data.clone()) {
                let tlcs_store = ctx.get_mutable_kv_store(&self.store_key);
                let chain_data: RawMsgContribution = msg.to_owned().into();
                tlcs_store.set(store_key.into(), chain_data.encode_to_vec());
            } else {
                return Err(AppError::InvalidRequest(
                    "The contribution data is invalid for the given round".into(),
                ));
            }
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

        //if loe_signature_is_valid(msg.round, msg.randomness.clone(), msg.signature.clone()) {
        if loe_signature_is_valid(msg.round, msg.signature.clone()) {
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
}

// {"public_key":"a0b862a7527fee3a731bcb59280ab6abd62d5c0b6ea03dc4ddf6612fdfc9d01f01c31542541771903475eb1ec6615f8d0df0b8b6dce385811d6dcf8cbefb8759e5e616a3dfd054c928940766d9a5b9db91e3b697e5d70a975181e007f87fca5e","period":3,"genesis_time":1677685200,"hash":"dbd506d6ef76e5f386f41c651dcb808c5bcbd75471cc4eafa3f4df7ad4e4c493","groupHash":"a81e9d63f614ccdb144b8ff79fbd4d5a2d22055c0bfe4ee9a8092003dab1c6c0","schemeID":"bls-unchained-on-g1","metadata":{"beaconID":"fastnet"}}
// const LOE_PUBLIC_KEY: String = "a0b862a7527fee3a731bcb59280ab6abd62d5c0b6ea03dc4ddf6612fdfc9d01f01c31542541771903475eb1ec6615f8d0df0b8b6dce385811d6dcf8cbefb8759e5e616a3dfd054c928940766d9a5b9db91e3b697e5d70a975181e007f87fca5e";
// The LOE data is from https://api.drand.sh/dbd506d6ef76e5f386f41c651dcb808c5bcbd75471cc4eafa3f4df7ad4e4c493/public/latest
// or (where round = 3276594)
// The LOE data is from https://api.drand.sh/dbd506d6ef76e5f386f41c651dcb808c5bcbd75471cc4eafa3f4df7ad4e4c493/public/{round}
// {"round":3276594,"randomness":"f282310f131ed63e0342cd7e47f9e4317b20fb6f652b03ce81378cf825227212","signature":"86f91b1eec7b22ecce1385ec1cc4861f43507fa897cad686e44a87986a7ce18a94fa7128d6f76d6b950bb4e559472539"}

fn loe_signature_is_valid(round: u64, signature: Vec<u8>) -> bool {
    let pk2 = G2Pubkey::from_fixed(LOE_PUBLIC_KEY).unwrap();

    // See https://api.drand.sh/dbd506d6ef76e5f386f41c651dcb808c5bcbd75471cc4eafa3f4df7ad4e4c493/public/1 for example data of the three inputs
    let nil: &mut [u8] = &mut [];

    /*
    let randomness_check = derive_randomness(&signature);
    if !(randomness == randomness_check) {
        return false;
    }
    */

    //match verify(&pk2, round, &nil, &hex_signature) {
    match verify(&pk2, round, &nil, &signature) {
        Err(_err) => return false,
        Ok(valid) => {
            if valid {
                return true;
            } else {
                return false;
            }
        }
    }
}

#[test]
fn test_round_signature() {
    //let signature: String = "9544ddce2fdbe8688d6f5b4f98eed5d63eee3902e7e162050ac0f45905a55657714880adabe3c3096b92767d886567d0".to_string();
    //let round: u32 = 1;
    //let randomness: Vec<u8> = hex::decode("f282310f131ed63e0342cd7e47f9e4317b20fb6f652b03ce81378cf825227212").unwrap();
    let signature: Vec<u8> = hex::decode("86f91b1eec7b22ecce1385ec1cc4861f43507fa897cad686e44a87986a7ce18a94fa7128d6f76d6b950bb4e559472539").unwrap();
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
    assert!(loe_signature_is_valid(round, signature));
}
