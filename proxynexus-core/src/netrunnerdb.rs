use crate::card_source::{CardSource, NrdbUrl};
use crate::card_store::CardStore;
use crate::error::{ProxyNexusError, Result};
use crate::models::CardRequest;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct NrdbResponse {
    data: Vec<NrdbDeck>,
}

#[derive(Debug, Deserialize)]
struct NrdbDeck {
    cards: HashMap<String, u32>,
}

impl CardSource for NrdbUrl {
    async fn to_card_requests(&self, store: &mut CardStore<'_>) -> Result<Vec<CardRequest>> {
        let codes = fetch_codes_from_nrdb_url(&self.0).await?;
        store.resolve_codes_to_card_requests(&codes).await
    }
}

async fn fetch_codes_from_nrdb_url(url: &str) -> Result<HashMap<String, u32>> {
    let (deck_id, api_path) = parse_nrdb_url(url)?;

    let api_url = format!(
        "https://netrunnerdb.com/api/2.0/public/{}/{}",
        api_path, deck_id
    );

    let response: NrdbResponse = {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let http_response = reqwest::get(&api_url).await?;

            if !http_response.status().is_success() {
                return Err(ProxyNexusError::Internal(format!(
                    "NetrunnerDB returned error: {}",
                    http_response.status()
                )));
            }

            http_response.json().await?
        }

        #[cfg(target_arch = "wasm32")]
        {
            let http_response = gloo_net::http::Request::get(&api_url)
                .send()
                .await?;

            if !http_response.ok() {
                return Err(ProxyNexusError::Internal(format!(
                    "NetrunnerDB returned error: {}",
                    http_response.status()
                )));
            }

            http_response.json().await?
        }
    };

    let cards = response
        .data
        .into_iter()
        .next()
        .ok_or_else(|| ProxyNexusError::Internal("Empty response from NetrunnerDB".into()))?
        .cards;

    Ok(cards)
}

fn parse_nrdb_url(url: &str) -> Result<(String, String)> {
    if url.contains("/decklist/") {
        let deck_id = url
            .split("/decklist/")
            .nth(1)
            .ok_or_else(|| ProxyNexusError::Internal("Invalid decklist URL".into()))?
            .split('/')
            .next()
            .ok_or_else(|| ProxyNexusError::Internal("Invalid decklist URL".into()))?
            .to_string();
        Ok((deck_id, "decklist".to_string()))
    } else if url.contains("/deck/view/") {
        let deck_id = url
            .split("/deck/view/")
            .nth(1)
            .ok_or_else(|| ProxyNexusError::Internal("Invalid deck URL".into()))?
            .trim_end_matches('/')
            .to_string();
        Ok((deck_id, "deck".to_string()))
    } else {
        Err(ProxyNexusError::Internal(
            "URL must be a NetrunnerDB decklist or deck URL".into(),
        ))
    }
}
