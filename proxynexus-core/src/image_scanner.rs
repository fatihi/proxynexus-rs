use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct ScannedImage {
    pub code: String,
    pub variant: String,
    pub path: PathBuf,
}

pub fn scan_images(dir: &Path) -> Vec<ScannedImage> {
    let mut images = Vec::new();

    for entry in WalkDir::new(dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        if let Some(ext) = path.extension() {
            let ext_lower = ext.to_string_lossy().to_lowercase();
            if ext_lower != "jpg" && ext_lower != "jpeg" {
                continue;
            }
        } else {
            continue;
        }

        if let Some((code, variant)) = parse_filename(path) {
            images.push(ScannedImage {
                code,
                variant,
                path: path.to_path_buf(),
            });
        }
    }

    images
}

fn parse_filename(path: &Path) -> Option<(String, String)> {
    let stem = path.file_stem()?.to_str()?;
    let mut parts = stem.splitn(2, '_');

    let code = parts.next()?;
    let variant = parts.next().unwrap_or("original");

    Some((code.to_string(), variant.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_filename_default() {
        let path = PathBuf::from("21001.jpg");
        let result = parse_filename(&path);
        assert_eq!(result, Some(("21001".to_string(), "original".to_string())));
    }

    #[test]
    fn test_parse_filename_alt() {
        let path = PathBuf::from("21001_alt1.jpg");
        let result = parse_filename(&path);
        assert_eq!(result, Some(("21001".to_string(), "alt1".to_string())));
    }

    #[test]
    fn test_parse_filename_rear() {
        let path = PathBuf::from("30212_rear.jpg");
        let result = parse_filename(&path);
        assert_eq!(result, Some(("30212".to_string(), "rear".to_string())));
    }
}
