use bytes::Bytes;
use database::DB;
use prost::Message;
//use proto_types::AccAddress;

use proto_messages::azkr::tlcs::v1beta1::{
    RawMsgContribution,
    MsgContribution,
    QueryAllContributionsResponse,
    RawMsgKeyPair,
    MsgKeyPair,
    QueryAllKeyPairsResponse,
    RawMsgLoeData,
    MsgLoeData,
    QueryAllLoeDataResponse,
};

use crate::{
    error::AppError,
    store::Store,
    types::{Context, QueryContext},
};

use crate::x::tlcs::crypto::{
    verify_participant_data,
};

// For LOE data verification
use hex_literal::hex;
use drand_verify::{
    derive_randomness,
    verify,
    G2Pubkey,
    Pubkey
};

const PARTICIPANT_DATA_KEY: [u8; 1] = [1];
const KEYPAIR_DATA_KEY: [u8; 1] = [2];
const LOE_DATA_KEY: [u8; 1] = [3];
const TMP_SCHEME_ID: [u8; 1] = [1];
const LOE_PUBLIC_KEY: [u8; 96] = hex!("a0b862a7527fee3a731bcb59280ab6abd62d5c0b6ea03dc4ddf6612fdfc9d01f01c31542541771903475eb1ec6615f8d0df0b8b6dce385811d6dcf8cbefb8759e5e616a3dfd054c928940766d9a5b9db91e3b697e5d70a975181e007f87fca5e");

// Key for KV store for contributions is vector of [PARTICIPANT_DATA_KEY, Round, scheme, address]
/*
pub fn build_data_key(round: u8, scheme: Vec<u8>, addr: AccAddress) -> Vec<u8> {
    let mut prefix = PARTICIPANT_DATA_KEY.to_vec();
    prefix.append(&mut round.to_le_bytes().to_vec());
    prefix.append(&mut scheme.to_vec());

    let mut addr: Vec<u8> = addr.into();

    prefix.append(addr.to_vec());
                                           
    return prefix;
}
*/

pub fn append_contribution<T: DB>(
    ctx: &mut Context<T>,
    msg: &MsgContribution,
) -> Result<(), AppError> {
    let tlcs_store = ctx.get_mutable_kv_store(Store::Tlcs);
    //let store_key = build_data_key(msg.round, TMP_SCHEME_ID, msg.address);
    
    let mut store_key = PARTICIPANT_DATA_KEY.to_vec();
    store_key.append(&mut msg.round.to_le_bytes().to_vec());
    store_key.append(&mut TMP_SCHEME_ID.to_vec());

    let addr: Vec<u8> = msg.address.clone().into();
    store_key.append(&mut addr.to_vec());

    // TODO Check for closed round

    //let verified = verify_participant_data(msg.round, msg.data.clone());
    if verify_participant_data(msg.round, msg.data.clone()) {
        tlcs_store.set(store_key.into(), msg.data.encode_to_vec());
    } else {
        return Err(AppError::InvalidRequest(
                   "the provided data is invalid for the given round".into(),
               ));
    }

    Ok(())
}

pub fn query_all_contributions<T: DB>(
    ctx: &QueryContext<T>,
) -> QueryAllContributionsResponse {
    let tlcs_store = ctx.get_kv_store(Store::Tlcs);
    let store_key = PARTICIPANT_DATA_KEY.to_vec();

    let all_raw_data = tlcs_store.range(store_key..);

    let mut contributions = vec![];

    for (_, row) in all_raw_data {
        /* Maybe the right way
        let msg = MsgContribution::decode::<Bytes>(msg.value.clone().into())
                        .map_err(|e| Error::DecodeGeneral(e.to_string()))?;
                    messages.push(Msg::Participate(msg));
                    */
        let contribution: RawMsgContribution = RawMsgContribution::decode::<Bytes>(row.into())
            .expect("invalid data in database - possible database corruption");
        contributions.push(contribution);
    }

    QueryAllContributionsResponse { contributions }
}

pub fn query_contributions_by_round<T: DB>(
    ctx: &QueryContext<T>,
    round: u32,
) -> QueryAllContributionsResponse {
    let tlcs_store = ctx.get_kv_store(Store::Tlcs);

    let mut store_key = PARTICIPANT_DATA_KEY.to_vec();
    store_key.append(&mut round.to_le_bytes().to_vec());

    let all_raw_data = tlcs_store.range(store_key..);

    let mut contributions = vec![];

    for (_, row) in all_raw_data {
        let contribution: RawMsgContribution = RawMsgContribution::decode::<Bytes>(row.into())
            .expect("invalid data in database - possible database corruption");
        contributions.push(contribution);

    }
    QueryAllContributionsResponse {
        contributions,
    }
}

pub fn query_contributions_by_round_and_scheme<T: DB>(
    ctx: &QueryContext<T>,
    round: u32,
    _scheme: u32, // TODO: make use of this
) -> QueryAllContributionsResponse {
    let tlcs_store = ctx.get_kv_store(Store::Tlcs);
    let mut store_key = PARTICIPANT_DATA_KEY.to_vec();
    store_key.append(&mut round.to_le_bytes().to_vec());
    store_key.append(&mut TMP_SCHEME_ID.to_vec());

    let all_raw_data = tlcs_store.range(store_key..);

    let mut contributions = vec![];

    for (_, row) in all_raw_data {
        let contribution: RawMsgContribution = RawMsgContribution::decode::<Bytes>(row.into())
            .expect("invalid data in database - possible database corruption");
        contributions.push(contribution);

    }
    QueryAllContributionsResponse {
        contributions,
    }
}

// Keypair section

// TODO maybe use this from the endblocker so all of the save/query functions are in the keeper
#[allow(dead_code)]
pub fn append_keypair<T: DB>(
    ctx: &mut Context<T>,
    msg: &MsgKeyPair,
) -> Result<(), AppError> {
    let tlcs_store = ctx.get_mutable_kv_store(Store::Tlcs);
    let mut prefix = LOE_DATA_KEY.to_vec();
    prefix.append(&mut msg.round.to_le_bytes().to_vec());
    prefix.append(&mut TMP_SCHEME_ID.to_vec());
    let raw = tlcs_store.get(&prefix);

    let mut all_responses = match raw {
        Some(raw) => QueryAllKeyPairsResponse::decode::<Bytes>(raw.into())
            .expect("invalid data in database - possible database corruption"),
        None => QueryAllKeyPairsResponse {
            keypairs: vec![],
        },
    };

    all_responses.keypairs.push(msg.to_owned().into());
    tlcs_store.set(KEYPAIR_DATA_KEY.into(), all_responses.encode_to_vec());

    Ok(())
}

pub fn query_all_keypairs<T: DB>(
    ctx: &QueryContext<T>,
) -> QueryAllKeyPairsResponse {
    let tlcs_store = ctx.get_kv_store(Store::Tlcs);
    let store_key = KEYPAIR_DATA_KEY.to_vec();

    let all_raw_data = tlcs_store.range(store_key..);

    let mut keypairs = vec![];

    for (_, row) in all_raw_data {
        let keypair: RawMsgKeyPair = RawMsgKeyPair::decode::<Bytes>(row.into())
            .expect("invalid data in database - possible database corruption");
        keypairs.push(keypair);

    }
    QueryAllKeyPairsResponse {
        keypairs,
    }
}

pub fn query_keypairs_by_round<T: DB>(
    ctx: &QueryContext<T>,
    round: u32,
) -> QueryAllKeyPairsResponse {
    let tlcs_store = ctx.get_kv_store(Store::Tlcs);
    let mut store_key = KEYPAIR_DATA_KEY.to_vec();
    store_key.append(&mut round.to_le_bytes().to_vec());
    let all_raw_data = tlcs_store.range(store_key..);

    let mut keypairs = vec![];

    for (_, row) in all_raw_data {
        let keypair: RawMsgKeyPair = RawMsgKeyPair::decode::<Bytes>(row.into())
            .expect("invalid data in database - possible database corruption");
        keypairs.push(keypair);

    }
    QueryAllKeyPairsResponse {
        keypairs,
    }
}

pub fn query_keypairs_by_round_and_scheme<T: DB>(
    ctx: &QueryContext<T>,
    round: u32,
    _scheme: u32, // TODO: make use of this
) -> QueryAllKeyPairsResponse {
    let tlcs_store = ctx.get_kv_store(Store::Tlcs);
    let mut store_key = KEYPAIR_DATA_KEY.to_vec();
    store_key.append(&mut round.to_le_bytes().to_vec());
    store_key.append(&mut TMP_SCHEME_ID.to_vec());
    let all_raw_data = tlcs_store.range(store_key..);

    let mut keypairs = vec![];

    for (_, row) in all_raw_data {
        let keypair: RawMsgKeyPair = RawMsgKeyPair::decode::<Bytes>(row.into())
            .expect("invalid data in database - possible database corruption");
        keypairs.push(keypair);
    }

    QueryAllKeyPairsResponse { keypairs }
}

pub fn append_loe_data<T: DB>(
    ctx: &mut Context<T>,
    msg: &MsgLoeData,
) -> Result<(), AppError> {
    let tlcs_store = ctx.get_mutable_kv_store(Store::Tlcs);
    
    let mut store_key = LOE_DATA_KEY.to_vec();
    store_key.append(&mut msg.round.to_le_bytes().to_vec());

    // TODO Check for closed round
    
    if loe_signature_is_valid(msg.round, msg.randomness.clone(), msg.signature.clone()) {
        //tlcs_store.set(store_key.into(), msg.randomness.encode_to_vec());
        tlcs_store.set(store_key.into(), <MsgLoeData as Into<RawMsgLoeData>>::into(msg.to_owned()).encode_to_vec());
    } else {
        return Err(AppError::InvalidRequest(
                   "the provided data is invalid for the given round".into(),
               ));
    }

    Ok(())
}

pub fn query_all_loe_data<T: DB>(
    ctx: &QueryContext<T>,
) -> QueryAllLoeDataResponse {
    let tlcs_store = ctx.get_kv_store(Store::Tlcs);
    let store_key = LOE_DATA_KEY.to_vec();
    let all_raw_data = tlcs_store.range(store_key..);

    let mut randomnesses = vec![];

    for (_, row) in all_raw_data {
        let loe_data: RawMsgLoeData = RawMsgLoeData::decode::<Bytes>(row.into())
            .expect("invalid data in database - possible database corruption");
        randomnesses.push(loe_data);
    }

    QueryAllLoeDataResponse { randomnesses }
}

pub fn query_loe_data_by_round<T: DB>(
    ctx: &QueryContext<T>,
    round: u32,
) -> QueryAllLoeDataResponse {
    let tlcs_store = ctx.get_kv_store(Store::Tlcs);

    let mut store_key = LOE_DATA_KEY.to_vec();
    store_key.append(&mut round.to_le_bytes().to_vec());

    let all_raw_data = tlcs_store.range(store_key..);

    let mut randomnesses = vec![];

    for (_, row) in all_raw_data {
        let loe_data: RawMsgLoeData = RawMsgLoeData::decode::<Bytes>(row.into())
            .expect("invalid data in database - possible database corruption");
        randomnesses.push(loe_data);

    }

    QueryAllLoeDataResponse { randomnesses }
}

// The LOE data is from https://api.drand.sh/dbd506d6ef76e5f386f41c651dcb808c5bcbd75471cc4eafa3f4df7ad4e4c493/info
// {"public_key":"a0b862a7527fee3a731bcb59280ab6abd62d5c0b6ea03dc4ddf6612fdfc9d01f01c31542541771903475eb1ec6615f8d0df0b8b6dce385811d6dcf8cbefb8759e5e616a3dfd054c928940766d9a5b9db91e3b697e5d70a975181e007f87fca5e","period":3,"genesis_time":1677685200,"hash":"dbd506d6ef76e5f386f41c651dcb808c5bcbd75471cc4eafa3f4df7ad4e4c493","groupHash":"a81e9d63f614ccdb144b8ff79fbd4d5a2d22055c0bfe4ee9a8092003dab1c6c0","schemeID":"bls-unchained-on-g1","metadata":{"beaconID":"fastnet"}}
// const LOE_PUBLIC_KEY: String = "a0b862a7527fee3a731bcb59280ab6abd62d5c0b6ea03dc4ddf6612fdfc9d01f01c31542541771903475eb1ec6615f8d0df0b8b6dce385811d6dcf8cbefb8759e5e616a3dfd054c928940766d9a5b9db91e3b697e5d70a975181e007f87fca5e";
// The LOE data is from https://api.drand.sh/dbd506d6ef76e5f386f41c651dcb808c5bcbd75471cc4eafa3f4df7ad4e4c493/public/latest
// or (where round = 3276594)
// The LOE data is from https://api.drand.sh/dbd506d6ef76e5f386f41c651dcb808c5bcbd75471cc4eafa3f4df7ad4e4c493/public/{round}
// {"round":3276594,"randomness":"f282310f131ed63e0342cd7e47f9e4317b20fb6f652b03ce81378cf825227212","signature":"86f91b1eec7b22ecce1385ec1cc4861f43507fa897cad686e44a87986a7ce18a94fa7128d6f76d6b950bb4e559472539"}

fn loe_signature_is_valid(
    round: u32,
    randomness: Vec<u8>,
    signature: Vec<u8>,
) -> bool {
    let pk2 = G2Pubkey::from_fixed(LOE_PUBLIC_KEY).unwrap();

    // See https://api.drand.sh/dbd506d6ef76e5f386f41c651dcb808c5bcbd75471cc4eafa3f4df7ad4e4c493/public/1 for example data of the three inputs
    let nil: &mut [u8] = &mut [];
    // TODO get rid of unwrap because Kevin hates them
    let hex_signature = hex::decode(signature).unwrap();
    let hex_randomness = hex::decode(randomness).unwrap();

    let randomness_check = derive_randomness(&hex_signature);
    if !(hex_randomness == randomness_check) {
       return false
    }

    match verify(&pk2, round as u64, &nil, &hex_signature) {
        Err(_err) => {
            return false
        }
        Ok(valid) => {
            if valid {
                return true
            } else {
                return false
            }
        }
    }
}

#[test]
fn test_round_signature() {
    //let signature: String = "9544ddce2fdbe8688d6f5b4f98eed5d63eee3902e7e162050ac0f45905a55657714880adabe3c3096b92767d886567d0".to_string();
    //let round: u32 = 1;
    let randomness: Vec<u8> = "f282310f131ed63e0342cd7e47f9e4317b20fb6f652b03ce81378cf825227212".into();
    let signature: Vec<u8> = "86f91b1eec7b22ecce1385ec1cc4861f43507fa897cad686e44a87986a7ce18a94fa7128d6f76d6b950bb4e559472539".into();
    let round: u32 = 3276594;
    assert!(loe_signature_is_valid(round, randomness, signature));

}
