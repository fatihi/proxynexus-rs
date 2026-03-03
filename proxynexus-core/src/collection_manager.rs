use crate::models::Manifest;
use dirs;
use std::fs;
use std::path::{Path, PathBuf};
use turso::{Connection, params};
use zip::ZipArchive;

pub struct CollectionManager {
    collections_dir: PathBuf,
    conn: Connection,
}

impl CollectionManager {
    pub fn new(conn: Connection) -> Result<Self, Box<dyn std::error::Error>> {
        let home = dirs::home_dir().ok_or("Could not find home directory")?;

        let proxynexus_dir = home.join(".proxynexus");
        let collections_dir = proxynexus_dir.join("collections");

        fs::create_dir_all(&collections_dir)?;

        Ok(Self {
            collections_dir,
            conn,
        })
    }

    pub async fn add_collection(
        &mut self,
        pnx_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !pnx_path.exists() {
            return Err(format!("File not found: {:?}", pnx_path).into());
        }

        let temp_dir = tempfile::tempdir()?;
        let temp_path = temp_dir.path();

        let file = fs::File::open(pnx_path)?;
        let mut archive = ZipArchive::new(file)?;
        archive.extract(temp_path)?;

        let manifest_path = temp_path.join("manifest.toml");
        let manifest_content = fs::read_to_string(&manifest_path)?;
        let manifest: Manifest = toml::from_str(&manifest_content)?;

        let collection_name = pnx_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or("Invalid filename")?
            .to_string();

        println!(
            "Adding collection: {} (v{}, {})",
            collection_name, manifest.version, manifest.language
        );

        let row = self
            .conn
            .query(
                "SELECT id FROM collections WHERE name = ?1",
                params![collection_name.clone()],
            )
            .await?
            .next()
            .await?;

        if row.is_some() {
            return Err(format!("Collection '{}' has already been added.", collection_name).into());
        }

        self.conn
            .execute(
                "INSERT INTO collections (name, version, language, added_date)
             VALUES (?1, ?2, ?3, datetime('now'))",
                params![collection_name.clone(), manifest.version, manifest.language,],
            )
            .await?;

        let collection_id = self.conn.last_insert_rowid();

        let collection_dir = self.collections_dir.join(collection_name.clone());
        fs::create_dir_all(&collection_dir)?;

        let src_images = temp_path.join("images");

        let mut printings_added = 0;
        let tx = self.conn.transaction().await?;

        for entry in fs::read_dir(&src_images)? {
            let entry = entry?;
            let path = entry.path();

            let (card_code, variant) = match Self::parse_filename(&path) {
                Some(parsed) => parsed,
                None => continue,
            };

            let file_name = path.file_name().unwrap().to_string_lossy();
            let file_path = format!("{}/{}", collection_name, file_name);

            tx.execute(
                "INSERT INTO printings (collection_id, card_code, variant, file_path)
                 VALUES (?1, ?2, ?3, ?4)",
                params![collection_id, card_code, variant, file_path,],
            )
            .await?;

            let dst_path = collection_dir.join(path.file_name().unwrap());
            fs::copy(entry.path(), dst_path)?;

            printings_added += 1;
        }

        tx.commit().await?;

        println!("Added {} printings", printings_added);
        println!("Collection '{}' added successfully!", collection_name);

        Ok(())
    }

    fn parse_filename(path: &Path) -> Option<(String, String)> {
        let stem = path.file_stem()?.to_str()?;

        let (code, variant) = if let Some((c, v)) = stem.split_once('_') {
            (c, v.to_lowercase())
        } else {
            (stem, "original".to_string())
        };

        if !code.chars().all(|c| c.is_ascii_digit()) {
            return None;
        }

        Some((code.to_string(), variant))
    }

    pub async fn get_collections(
        &self,
    ) -> Result<Vec<(String, String, String)>, Box<dyn std::error::Error>> {
        let mut stmt = self
            .conn
            .prepare("SELECT name, version, language FROM collections ORDER BY name")
            .await?;
        let mut rows = stmt.query(()).await?;
        let mut results = Vec::new();

        while let Some(row) = rows.next().await? {
            results.push((row.get(0)?, row.get(1)?, row.get(2)?));
        }

        Ok(results)
    }

    pub async fn collection_exists(&self, name: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let row = self
            .conn
            .query(
                "SELECT COUNT(*) FROM collections WHERE name = ?1",
                params![name],
            )
            .await?
            .next()
            .await?;

        let count: i64 = match row {
            Some(r) => r.get(0)?,
            None => 0,
        };
        Ok(count > 0)
    }

    pub async fn remove_collection(
        &mut self,
        collection_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let row = self
            .conn
            .query(
                "SELECT id FROM collections WHERE name = ?1",
                params![collection_name],
            )
            .await?
            .next()
            .await?;

        let collection_id: i64 = match row {
            Some(r) => r.get(0)?,
            None => return Err(format!("Collection '{}' not found", collection_name).into()),
        };

        let tx = self.conn.transaction().await?;

        tx.execute(
            "DELETE FROM printings WHERE collection_id = ?1",
            params![collection_id],
        )
        .await?;

        tx.execute(
            "DELETE FROM collections WHERE id = ?1",
            params![collection_id],
        )
        .await?;

        tx.commit().await?;

        let collection_dir = self.collections_dir.join(collection_name);
        if collection_dir.exists() {
            fs::remove_dir_all(&collection_dir)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_parse_filename_variants() {
        assert_eq!(
            CollectionManager::parse_filename(Path::new("01001.jpg")),
            Some(("01001".to_string(), "original".to_string()))
        );

        assert_eq!(
            CollectionManager::parse_filename(Path::new("01001_alt1.jpg")),
            Some(("01001".to_string(), "alt1".to_string()))
        );

        assert_eq!(
            CollectionManager::parse_filename(Path::new("01001_Rear.png")),
            Some(("01001".to_string(), "rear".to_string()))
        );

        assert_eq!(
            CollectionManager::parse_filename(Path::new("notacode.jpg")),
            None
        );
        assert_eq!(
            CollectionManager::parse_filename(Path::new("abc_alt.jpg")),
            None
        );
    }
}
