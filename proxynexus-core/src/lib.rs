mod border_generator;
pub mod card_source;
pub mod card_store;
pub mod catalog;
pub mod collection_builder;
pub mod collection_manager;
mod db_schema;
mod models;
pub mod mpc;
pub mod netrunnerdb;
pub mod pdf;
pub mod query;

use turso::Connection;

pub async fn setup_database(conn: &Connection) -> Result<(), Box<dyn std::error::Error>> {
    conn.execute("PRAGMA foreign_keys = ON", ()).await?;
    db_schema::create_app_schema(conn).await?;
    Ok(())
}
