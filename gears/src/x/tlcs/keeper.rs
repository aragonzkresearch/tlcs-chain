use bytes::Bytes;
use database::DB;
use prost::Message;
use proto_messages::azkr::tlcs::v1beta1::{
    MsgParticipantContribution, QueryAllParticipantsContributionsResponse,
};

use crate::{
    error::AppError,
    store::Store,
    types::{Context, QueryContext},
};

const PARTICIPANTS_KEY: [u8; 1] = [1];

pub fn append_participant_contribution<T: DB>(
    ctx: &mut Context<T>,
    msg: &MsgParticipantContribution,
) -> Result<(), AppError> {
    let tlcs_store = ctx.get_mutable_kv_store(Store::Tlcs);
    let raw = tlcs_store.get(&PARTICIPANTS_KEY);

    let mut all_responses = match raw {
        Some(raw) => QueryAllParticipantsContributionsResponse::decode::<Bytes>(raw.into())
            .expect("invalid data in database - possible database corruption"),
        None => QueryAllParticipantsContributionsResponse {
            contributions: vec![],
        },
    };

    all_responses.contributions.push(msg.to_owned().into());

    tlcs_store.set(PARTICIPANTS_KEY.into(), all_responses.encode_to_vec());

    Ok(())
}

pub fn query_all_participant_contributions<T: DB>(
    ctx: &QueryContext<T>,
) -> QueryAllParticipantsContributionsResponse {
    let tlcs_store = ctx.get_kv_store(Store::Tlcs);
    let raw = tlcs_store.get(&PARTICIPANTS_KEY);

    match raw {
        Some(raw) => QueryAllParticipantsContributionsResponse::decode::<Bytes>(raw.into())
            .expect("invalid data in database - possible database corruption"),
        None => QueryAllParticipantsContributionsResponse {
            contributions: vec![],
        },
    }
}
