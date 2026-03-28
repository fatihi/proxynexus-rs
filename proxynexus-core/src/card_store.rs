use crate::card_source::{CardSource, Cardlist, SetName};
use crate::db_storage::{DbStorage, build_in_clause, quote_sql_string};
use crate::models::{CardRequest, Printing, PrintingPart};
use gluesql::FromGlueRow;
use gluesql::core::row_conversion::SelectExt;
use gluesql::prelude::*;
use std::collections::{HashMap, HashSet};
use std::string::String;
use tracing::warn;

#[derive(FromGlueRow)]
struct PackRow {
    pack_name: String,
    pack_code: String,
    coll_name: Option<String>,
    coll_count: i64,
    date_release: Option<String>,
}

#[derive(FromGlueRow)]
struct CardNameRow {
    code: String,
    title: String,
    pack_code: String,
    title_normalized: String,
}

#[derive(FromGlueRow)]
struct CardRequestRow {
    code: String,
    title: String,
    quantity: i64,
    pack_code: String,
}

#[derive(FromGlueRow)]
struct CardRow {
    code: String,
    title: String,
}

#[derive(FromGlueRow)]
struct CardTitleRow {
    title: String,
}

#[derive(FromGlueRow)]
struct AvailablePrintingRow {
    title: String,
    code: String,
    variant: String,
    file_path: String,
    part: String,
    name: String,
    side: String,
    pack_code: String,
    date_release: Option<String>,
}

pub fn normalize_title(title: &str) -> String {
    deunicode::deunicode(title)
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

pub fn clean_card_name(name: &str) -> &str {
    name.trim_end_matches(|c: char| !c.is_alphanumeric() && !"!.*)\"'”’“‘".contains(c))
}

impl CardSource for Cardlist {
    async fn to_card_requests(
        &self,
        store: &mut CardStore<'_>,
    ) -> Result<Vec<CardRequest>, Box<dyn std::error::Error>> {
        let (requests, not_found) = store.parse_cardlist_into_card_requests(&self.0).await?;

        if !not_found.is_empty() {
            warn!(
                "{} card(s) not found in catalog: {:?}",
                not_found.len(),
                not_found
            );
            warn!("Consider running 'proxynexus catalog update'");
        }

        Ok(requests)
    }
}

impl CardSource for SetName {
    async fn to_card_requests(
        &self,
        store: &mut CardStore<'_>,
    ) -> Result<Vec<CardRequest>, Box<dyn std::error::Error>> {
        store.get_card_requests_from_set_name(&self.0).await
    }
}

pub struct CardStore<'a> {
    db: &'a mut DbStorage,
}

type CardOverride<'a> = (&'a str, Option<String>, Option<String>, Option<String>);

impl<'a> CardStore<'a> {
    pub fn new(db: &'a mut DbStorage) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self { db })
    }

    pub async fn get_all_card_names(&mut self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let query = "SELECT DISTINCT title FROM cards ORDER BY title";
        let payloads = self.db.execute(query).await?;
        let mut names = Vec::new();

        if let Some(payload) = payloads.into_iter().next() {
            names = payload
                .rows_as::<CardTitleRow>()?
                .into_iter()
                .map(|row| row.title)
                .collect();
        }

        Ok(names)
    }

    async fn parse_cardlist_into_card_requests(
        &mut self,
        text: &str,
    ) -> Result<(Vec<CardRequest>, Vec<String>), Box<dyn std::error::Error>> {
        type CardlistEntry<'a> = (&'a str, u32, Option<String>, Option<String>, Option<String>);
        let mut entries: Vec<CardlistEntry> = Vec::new();

        for line in text.lines() {
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }

            let (qty, rest) = Self::parse_quantity(line);
            let (name, variant_pref, collection_pref, pack_code_pref) =
                Self::parse_overrides(rest)?;

            let name = clean_card_name(name);
            entries.push((name, qty, variant_pref, collection_pref, pack_code_pref));
        }

        if entries.is_empty() {
            return Ok((Vec::new(), Vec::new()));
        }

        let titles: Vec<&str> = entries.iter().map(|(name, ..)| *name).collect();
        let (resolved_cards, not_found) = self.resolve_names_to_cards(&titles).await?;

        let mut requests = Vec::new();

        for (name, qty, variant, collection, requested_pack_code) in entries {
            if let Some((code, title, resolved_pack_code)) = resolved_cards.get(name) {
                requests.extend(std::iter::repeat_n(
                    CardRequest {
                        title: title.clone(),
                        code: code.clone(),
                        variant: variant.clone(),
                        collection: collection.clone(),
                        pack_code: requested_pack_code
                            .clone()
                            .or_else(|| Some(resolved_pack_code.clone())),
                    },
                    qty as usize,
                ));
            }
        }

        Ok((requests, not_found))
    }

    pub fn parse_quantity(line: &str) -> (u32, &str) {
        if let Some((qty_str, card_name)) = line
            .split_once("x ")
            .filter(|(qty_str, _)| qty_str.chars().all(|c| c.is_ascii_digit()))
        {
            let qty: u32 = qty_str.parse().unwrap_or(1);
            (qty, card_name.trim())
        } else if let Some((prefix, rest)) = line.split_once(' ') {
            if prefix.chars().all(|c| c.is_ascii_digit()) {
                let qty: u32 = prefix.parse().unwrap_or(1);
                (qty, rest.trim())
            } else {
                (1, line)
            }
        } else {
            (1, line)
        }
    }

    pub fn parse_overrides(text: &str) -> Result<CardOverride<'_>, Box<dyn std::error::Error>> {
        if let Some(bracket_start) = text.find('[') {
            let name = text[..bracket_start].trim();
            let bracket_end = text.find(']').ok_or("Unclosed bracket in card override")?;

            let inner = &text[bracket_start + 1..bracket_end];
            if inner.trim().is_empty() {
                return Err("Empty override brackets".into());
            }

            let parts: Vec<Option<String>> = inner
                .split(':')
                .map(|s| {
                    let cleaned = s.trim().to_lowercase();
                    if cleaned.is_empty() {
                        None
                    } else {
                        Some(cleaned)
                    }
                })
                .collect();

            let variant = parts.first().cloned().flatten();
            let collection = parts.get(1).cloned().flatten();
            let pack_code = parts.get(2).cloned().flatten();

            Ok((name, variant, collection, pack_code))
        } else {
            Ok((text.trim(), None, None, None))
        }
    }

    async fn resolve_names_to_cards(
        &mut self,
        names: &[&str],
    ) -> Result<(HashMap<String, (String, String, String)>, Vec<String>), Box<dyn std::error::Error>>
    {
        let normalized_name_map: HashMap<&str, String> = names
            .iter()
            .map(|&name| (name, normalize_title(name)))
            .collect();

        let unique_normalized_name: HashSet<&str> =
            normalized_name_map.values().map(|s| s.as_str()).collect();
        let in_clause = build_in_clause(unique_normalized_name);

        let query = format!(
            "SELECT c.code, c.title, c.pack_code, c.title_normalized
             FROM cards c
             JOIN packs p ON c.pack_code = p.code
             WHERE c.title_normalized IN ({})
             ORDER BY
                 CASE WHEN p.date_release IS NULL THEN 1 ELSE 0 END,
                 p.date_release DESC",
            in_clause
        );

        let payloads = self.db.execute(&query).await?;
        let mut resolved_map: HashMap<String, (String, String, String)> = HashMap::new();

        if let Some(payload) = payloads.into_iter().next() {
            let name_rows = payload.rows_as::<CardNameRow>()?;
            for row in name_rows {
                resolved_map.entry(row.title_normalized).or_insert((
                    row.code,
                    row.title,
                    row.pack_code,
                ));
            }
        }

        if resolved_map.is_empty() && !names.is_empty() {
            return Err(
                "No card titles found in the local catalog. Is your catalog seeded?".into(),
            );
        }

        let mut title_to_card: HashMap<String, (String, String, String)> = HashMap::new();
        let mut not_found = Vec::new();

        for (&title, normalized) in &normalized_name_map {
            if let Some(card_data) = resolved_map.get(normalized) {
                title_to_card.insert(title.to_string(), card_data.clone());
            } else {
                not_found.push(title.to_string());
            }
        }

        Ok((title_to_card, not_found))
    }

    pub async fn get_available_packs(
        &mut self,
    ) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
        let query = "
            SELECT
                p.name as pack_name,
                p.code as pack_code,
                col.name as coll_name,
                COUNT(pr.card_code) as coll_count,
                p.date_release
            FROM packs p
            JOIN cards c ON c.pack_code = p.code
            LEFT JOIN printings pr ON pr.card_code = c.code
            LEFT JOIN collections col ON pr.collection_id = col.id
            GROUP BY p.code, col.id
        ";

        let payloads = self.db.execute(query).await?;

        struct PackGroup {
            name: String,
            date_release: String,
            collections: Vec<String>,
        }

        let mut pack_data: HashMap<String, PackGroup> = HashMap::new();

        if let Some(payload) = payloads.into_iter().next() {
            let pack_rows = payload.rows_as::<PackRow>()?;

            for row in pack_rows {
                let date_release = row.date_release.unwrap_or_default();

                let entry = pack_data.entry(row.pack_code).or_insert_with(|| PackGroup {
                    name: row.pack_name,
                    date_release,
                    collections: Vec::new(),
                });

                if let Some(name) = row.coll_name
                    && row.coll_count > 0
                {
                    entry
                        .collections
                        .push(format!("{} in {}", row.coll_count, name));
                }
            }
        }

        let mut sorted_packs: Vec<_> = pack_data.into_values().collect();
        sorted_packs.sort_by(|a, b| a.date_release.cmp(&b.date_release));

        let mut results = Vec::new();

        for mut pack in sorted_packs {
            pack.collections.sort();
            let meta = if pack.collections.is_empty() {
                None
            } else {
                Some(pack.collections.join(", "))
            };

            let display_meta = meta
                .map(|m| format!("# {}", m))
                .unwrap_or_else(|| "# no printings available".to_string());

            results.push((pack.name, display_meta));
        }

        Ok(results)
    }

    async fn get_card_requests_from_set_name(
        &mut self,
        set_name: &str,
    ) -> Result<Vec<CardRequest>, Box<dyn std::error::Error>> {
        let query = format!(
            "SELECT c.code, c.title, c.quantity, c.pack_code
             FROM cards c
             JOIN packs p ON c.pack_code = p.code
             WHERE LOWER(p.name) = {}
             ORDER BY c.code",
            quote_sql_string(&set_name.to_lowercase())
        );

        let payloads = self.db.execute(&query).await?;
        let mut results = Vec::new();

        if let Some(payload) = payloads.into_iter().next() {
            let request_rows = payload.rows_as::<CardRequestRow>()?;

            for row in request_rows {
                results.extend(std::iter::repeat_n(
                    CardRequest {
                        title: row.title,
                        code: row.code,
                        variant: None,
                        collection: None,
                        pack_code: Some(row.pack_code),
                    },
                    row.quantity as usize,
                ));
            }
        }

        if results.is_empty() {
            return Err(format!("No cards found for set '{}'", set_name).into());
        }

        Ok(results)
    }

    pub async fn resolve_codes_to_card_requests(
        &mut self,
        codes: &HashMap<String, u32>,
    ) -> Result<Vec<CardRequest>, Box<dyn std::error::Error>> {
        if codes.is_empty() {
            return Ok(Vec::new());
        }

        let in_clause = build_in_clause(codes.keys());

        let query = format!(
            "SELECT code, title FROM cards WHERE code IN ({})",
            in_clause
        );

        let payloads = self.db.execute(&query).await?;
        let mut resolved_titles = HashMap::new();

        if let Some(payload) = payloads.into_iter().next() {
            let card_rows = payload.rows_as::<CardRow>()?;
            for row in card_rows {
                resolved_titles.insert(row.code, row.title);
            }
        }

        if resolved_titles.is_empty() && !codes.is_empty() {
            return Err("No card codes found in the local catalog. Is your catalog seeded?".into());
        }

        let mut requests = Vec::new();
        for (code, qty) in codes {
            if let Some(title) = resolved_titles.get(code) {
                requests.extend(std::iter::repeat_n(
                    CardRequest {
                        title: title.clone(),
                        code: code.clone(),
                        variant: None,
                        collection: None,
                        pack_code: None,
                    },
                    *qty as usize,
                ));
            } else {
                warn!(
                    "Card code '{}' from NetrunnerDB not found in local catalog",
                    code
                );
                warn!("Consider running 'proxynexus catalog update'");
            }
        }

        Ok(requests)
    }

    pub async fn get_available_printings(
        &mut self,
        card_requests: &[CardRequest],
    ) -> Result<HashMap<String, Vec<Printing>>, Box<dyn std::error::Error>> {
        let unique_titles: HashSet<String> = card_requests
            .iter()
            .map(|r| normalize_title(&r.title))
            .collect();

        let in_clause = build_in_clause(&unique_titles);

        let query = format!(
            "SELECT c.title, c.code, p.variant, p.file_path, p.part, col.name, c.side, c.pack_code, pks.date_release
             FROM printings p
             JOIN cards c ON p.card_code = c.code
             JOIN collections col ON p.collection_id = col.id
             JOIN packs pks ON c.pack_code = pks.code
             WHERE c.title_normalized IN ({})",
            in_clause
        );

        let payloads = self.db.execute(&query).await?;
        let mut resolved_printings: HashMap<String, Vec<Printing>> = HashMap::new();

        if let Some(payload) = payloads.into_iter().next() {
            let printing_rows = payload.rows_as::<AvailablePrintingRow>()?;
            resolved_printings = Self::assemble_printings(printing_rows);
        }

        if resolved_printings.is_empty() && !card_requests.is_empty() {
            return Err("No printings found in your collections for any requested cards.".into());
        }

        let mut missing_titles = HashSet::new();
        for req in card_requests {
            let norm = normalize_title(&req.title);
            if !resolved_printings.contains_key(&norm) && missing_titles.insert(norm) {
                warn!(
                    "No printings found for '{}' in your collections.",
                    req.title
                );
            }
        }

        Ok(resolved_printings)
    }

    fn assemble_printings(rows: Vec<AvailablePrintingRow>) -> HashMap<String, Vec<Printing>> {
        let mut resolved_printings: HashMap<String, Vec<Printing>> = HashMap::new();
        let mut groups: HashMap<(String, String, String, String), Vec<AvailablePrintingRow>> =
            HashMap::new();

        for row in rows {
            let normalized = normalize_title(&row.title);
            let key = (
                normalized,
                row.code.clone(),
                row.variant.clone(),
                row.name.clone(),
            );
            groups.entry(key).or_default().push(row);
        }

        for ((normalized, code, variant, collection), rows) in groups {
            let mut image_key = String::new();
            let mut parts = Vec::new();

            let first_row = &rows[0];
            let title = first_row.title.clone();
            let side = first_row.side.clone();
            let pack_code = first_row.pack_code.clone();
            let date_release = first_row.date_release.clone();

            for row in rows {
                if row.part == "front" {
                    image_key = row.file_path;
                } else {
                    parts.push(PrintingPart {
                        name: row.part,
                        image_key: row.file_path,
                    });
                }
            }

            let printing = Printing {
                card_title: title,
                card_code: code,
                variant,
                image_key,
                parts,
                collection,
                side,
                pack_code,
                date_release,
            };

            resolved_printings
                .entry(normalized)
                .or_default()
                .push(printing);
        }

        for printings in resolved_printings.values_mut() {
            printings.sort_by_key(|p| (p.date_release.is_none(), p.date_release.clone()));
        }

        resolved_printings
    }

    pub fn resolve_printings(
        &self,
        requests: &[CardRequest],
        available: &HashMap<String, Vec<Printing>>,
    ) -> Result<Vec<Printing>, Box<dyn std::error::Error>> {
        let mut result = Vec::new();

        for request in requests {
            let normalized = normalize_title(&request.title);

            if let Some(printings) = available.get(&normalized) {
                match Self::select_printing(request, printings) {
                    Ok(printing) => result.push(printing),
                    Err(e) => {
                        warn!("{}", e);
                        if let Some(fallback) = printings.first() {
                            warn!("  Using: {} from {}", fallback.variant, fallback.collection);
                            result.push(fallback.clone());
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    pub fn select_printing(
        request: &CardRequest,
        printings: &[Printing],
    ) -> Result<Printing, Box<dyn std::error::Error>> {
        let mut candidates: Vec<&Printing> = printings.iter().collect();

        let target_variant = request.variant.as_deref().unwrap_or("original");

        candidates.sort_by_key(|p| {
            (
                p.variant != target_variant,
                p.card_code != request.code,
                request.collection.as_ref() != Some(&p.collection),
                request.pack_code.as_ref() != Some(&p.pack_code),
                p.date_release.is_none(),
                p.date_release.clone(),
            )
        });

        candidates
            .into_iter()
            .next()
            .cloned()
            .ok_or_else(|| format!("No matching printing found for '{}'", request.title).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Printing;

    fn mock_printing(
        code: &str,
        variant: &str,
        coll: &str,
        pack: &str,
        date: Option<&str>,
    ) -> Printing {
        Printing {
            card_title: "Sure Gamble".into(),
            card_code: code.into(),
            variant: variant.into(),
            image_key: format!("{}.jpg", code),
            parts: Vec::new(),
            collection: coll.into(),
            side: "runner".into(),
            pack_code: pack.into(),
            date_release: date.map(|s| s.to_string()),
        }
    }

    #[test]
    fn test_select_printing_prioritization() {
        let p1 = mock_printing("01050", "original", "ffg-en", "core", Some("2012-12-01"));
        let p2 = mock_printing("01050", "alt1", "standard", "core", Some("2012-12-01"));
        let p3 = mock_printing(
            "20050",
            "original",
            "alt-arts",
            "revised",
            Some("2017-01-01"),
        );
        let p_collection = mock_printing(
            "01050",
            "original",
            "alt-arts",
            "revised",
            Some("2017-01-01"),
        );

        let available = vec![p1.clone(), p2.clone(), p3.clone(), p_collection.clone()];

        // Exact variant match
        let req = CardRequest {
            title: "Sure Gamble".into(),
            code: "01050".into(),
            variant: Some("alt1".into()),
            collection: None,
            pack_code: None,
        };
        assert_eq!(
            CardStore::select_printing(&req, &available)
                .unwrap()
                .variant,
            "alt1"
        );

        // Exact collection match
        let req = CardRequest {
            title: "Sure Gamble".into(),
            code: "01050".into(),
            variant: None,
            collection: Some("alt-arts".into()),
            pack_code: None,
        };
        assert_eq!(
            CardStore::select_printing(&req, &available)
                .unwrap()
                .collection,
            "alt-arts"
        );

        // Exact pack match
        let req = CardRequest {
            title: "Sure Gamble".into(),
            code: "01050".into(),
            variant: None,
            collection: None,
            pack_code: Some("core".into()),
        };
        assert_eq!(
            CardStore::select_printing(&req, &available)
                .unwrap()
                .pack_code,
            "core"
        );

        // Variant Fallback: If 'core' original is missing, pick 'revised' original over 'core' alt
        let available_missing_core_orig = vec![p2.clone(), p3.clone()];
        let req = CardRequest {
            title: "Sure Gamble".into(),
            code: "01050".into(),
            variant: Some("original".into()),
            collection: None,
            pack_code: Some("core".into()),
        };
        let result = CardStore::select_printing(&req, &available_missing_core_orig).unwrap();
        assert_eq!(result.variant, "original");
        assert_eq!(result.pack_code, "revised");

        // Default to earliest original
        let req = CardRequest {
            title: "Sure Gamble".into(),
            code: "01050".into(),
            variant: None,
            collection: None,
            pack_code: None,
        };
        let result = CardStore::select_printing(&req, &available).unwrap();
        assert_eq!(result.variant, "original");
        assert_eq!(result.date_release, Some("2012-12-01".to_string()));

        // Variant Match beats Exact ID Match
        let p4_revised_alt =
            mock_printing("20050", "alt2", "standard", "revised", Some("2017-01-01"));
        let available_mixed = vec![p1.clone(), p4_revised_alt.clone()];
        let req = CardRequest {
            title: "Sure Gamble".into(),
            code: "20050".into(),
            variant: Some("original".into()),
            collection: None,
            pack_code: None,
        };
        let result = CardStore::select_printing(&req, &available_mixed).unwrap();
        assert_eq!(result.card_code, "01050");
        assert_eq!(result.variant, "original");
    }

    #[test]
    fn test_clean_card_name() {
        // valid trailing characters remain
        assert_eq!(clean_card_name("Snare!"), "Snare!");
        assert_eq!(clean_card_name("Eli 1.0"), "Eli 1.0");
        assert_eq!(
            clean_card_name("The World is Yours*"),
            "The World is Yours*"
        );
        assert_eq!(clean_card_name("Masterwork (v37)"), "Masterwork (v37)");
        assert_eq!(
            clean_card_name("\"Freedom Through Equality\""),
            "\"Freedom Through Equality\""
        );
        assert_eq!(
            clean_card_name("Title (with parens)"),
            "Title (with parens)"
        );

        // invalid trailing characters get stripped
        assert_eq!(clean_card_name("Hedge Fund ●"), "Hedge Fund");
        assert_eq!(clean_card_name("Sure Gamble -"), "Sure Gamble");
        assert_eq!(clean_card_name("Paperclip ●●●"), "Paperclip");
        assert_eq!(clean_card_name("Card Name ! ●"), "Card Name !");
        assert_eq!(clean_card_name("Card Name ●●●"), "Card Name");
    }

    #[test]
    fn test_parse_quantity() {
        assert_eq!(
            CardStore::parse_quantity("3x Sure Gamble"),
            (3, "Sure Gamble")
        );
        assert_eq!(
            CardStore::parse_quantity("3 Sure Gamble"),
            (3, "Sure Gamble")
        );
        assert_eq!(CardStore::parse_quantity("Sure Gamble"), (1, "Sure Gamble"));
        assert_eq!(
            CardStore::parse_quantity("10x Hedge Fund"),
            (10, "Hedge Fund")
        );
    }

    #[test]
    fn test_parse_overrides() {
        // Full override
        let (name, v, c, p) = CardStore::parse_overrides("Sure Gamble [alt:ffg-en:core]").unwrap();
        assert_eq!(name, "Sure Gamble");
        assert_eq!(v, Some("alt".to_string()));
        assert_eq!(c, Some("ffg-en".to_string()));
        assert_eq!(p, Some("core".to_string()));

        // Partial, variant only
        let (_, v, c, p) = CardStore::parse_overrides("Sure Gamble [alt]").unwrap();
        assert_eq!(v, Some("alt".to_string()));
        assert_eq!(c, None);
        assert_eq!(p, None);

        // Partial, skipped slots
        let (_, v, c, p) = CardStore::parse_overrides("Sure Gamble [:std:]").unwrap();
        assert_eq!(v, None);
        assert_eq!(c, Some("std".to_string()));
        assert_eq!(p, None);

        // Case normalization in overrides
        let (_, v, _, _) = CardStore::parse_overrides("Card [ALT]").unwrap();
        assert_eq!(v, Some("alt".to_string()));
    }
}
