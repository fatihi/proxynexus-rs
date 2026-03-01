use crate::db_schema;
use crate::models::{Card, Pack};
use serde::Deserialize;
use std::path::PathBuf;
use turso::{Builder, Connection, params};

const CARDS_JSON: &str = include_str!("../data/netrunnerdb_cards.json");
const PACKS_JSON: &str = include_str!("../data/netrunnerdb_packs.json");

#[derive(Debug, Deserialize)]
struct CardsResponse {
    data: Vec<Card>,
    last_updated: String,
}

#[derive(Debug, Deserialize)]
struct PacksResponse {
    data: Vec<Pack>,
}

pub fn normalize_title(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

pub struct Catalog {
    conn: Connection,
}

impl Catalog {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let home = dirs::home_dir().ok_or("Could not find home directory")?;
        let proxynexus_dir = home.join(".proxynexus");
        std::fs::create_dir_all(&proxynexus_dir)?;

        let app_db_path = proxynexus_dir
            .join("proxynexus.db")
            .to_string_lossy()
            .to_string();

        let db = Builder::new_local(&app_db_path).build().await?;
        let conn = db.connect()?;
        conn.execute("PRAGMA foreign_keys = ON", ()).await?;
        db_schema::create_app_schema(&conn).await?;

        Ok(Self { conn })
    }

    pub async fn seed_if_empty(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let count = self.get_card_count().await?;

        if count == 0 {
            println!("Seeding card catalog...");
            self.seed_from_json(CARDS_JSON, PACKS_JSON).await?;
            println!("Card catalog seeded successfully!");
        }

        Ok(())
    }

    pub async fn update_from_api(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let cards_json =
            reqwest::blocking::get("https://netrunnerdb.com/api/2.0/public/cards")?.text()?;

        let packs_json =
            reqwest::blocking::get("https://netrunnerdb.com/api/2.0/public/packs")?.text()?;

        self.seed_from_json(&cards_json, &packs_json).await?;

        Ok(())
    }

    pub async fn update_catalog_from_files(
        &mut self,
        cards_path: &PathBuf,
        packs_path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cards_json = std::fs::read_to_string(cards_path)?;
        let packs_json = std::fs::read_to_string(packs_path)?;

        self.seed_from_json(&cards_json, &packs_json).await?;

        Ok(())
    }

    pub async fn get_info(&self) -> Result<String, Box<dyn std::error::Error>> {
        let count = self.get_card_count().await?;

        let last_updated = self
            .conn
            .query("SELECT value FROM meta WHERE key = 'catalog_version'", ())
            .await?
            .next()
            .await?
            .map(|r| r.get(0))
            .transpose()?;

        let info = format!(
            "Card Catalog Info:\n\
         - Cards: {}\n\
         - Last Updated: {}",
            count,
            last_updated.unwrap_or_else(|| "Unknown (bundled snapshot)".to_string())
        );

        Ok(info)
    }

    async fn seed_from_json(
        &mut self,
        cards_json: &str,
        packs_json: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cards_response: CardsResponse = serde_json::from_str(cards_json)?;
        let packs_response: PacksResponse = serde_json::from_str(packs_json)?;

        let tx = self.conn.transaction().await?;

        tx.execute("DELETE FROM cards", ()).await?;
        tx.execute("DELETE FROM packs", ()).await?;

        for pack in packs_response.data {
            tx.execute(
                "INSERT INTO packs (code, name, date_release) VALUES (?1, ?2, ?3)",
                params![pack.code, pack.name, pack.date_release],
            )
            .await?;
        }

        for card in cards_response.data {
            tx.execute(
                "INSERT INTO cards (code, title, title_normalized, pack_code, side, quantity)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    card.code,
                    card.title.clone(),
                    normalize_title(&card.title),
                    card.pack_code,
                    card.side_code,
                    card.quantity,
                ],
            )
            .await?;
        }

        tx.execute(
            "INSERT OR REPLACE INTO meta (key, value) VALUES ('catalog_version', ?1)",
            params![cards_response.last_updated],
        )
        .await?;

        tx.commit().await?;

        Ok(())
    }

    async fn get_card_count(&self) -> Result<i64, Box<dyn std::error::Error>> {
        Ok(self
            .conn
            .query("SELECT COUNT(*) FROM cards", ())
            .await?
            .next()
            .await?
            .map(|r| r.get(0))
            .transpose()?
            .unwrap_or(0))
    }
}
