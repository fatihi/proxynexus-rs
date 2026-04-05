use crate::card_store::CardStore;
use crate::error::Result;
use crate::models::CardRequest;

pub trait CardSource {
    #![allow(async_fn_in_trait)]
    async fn to_card_requests(&self, store: &mut CardStore<'_>) -> Result<Vec<CardRequest>>;
}

pub struct Cardlist(pub String);
pub struct SetName(pub String);
pub struct NrdbUrl(pub String);
