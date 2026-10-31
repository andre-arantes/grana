use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::model::Data;

/// `$GRANA_FILE` if set (handy for testing), otherwise `~/.local/share/grana/grana.json`.
pub fn data_path() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("GRANA_FILE") {
        return Ok(PathBuf::from(path));
    }
    let home = std::env::var("HOME").context("HOME is not set")?;
    Ok(PathBuf::from(home).join(".local/share/grana/grana.json"))
}

pub fn load(path: &Path) -> Result<Data> {
    if !path.exists() {
        return Ok(Data::default());
    }
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}

/// Writes to a temp file then renames, so a crash never leaves a half-written file.
pub fn save(path: &Path, data: &Data) -> Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, serde_json::to_string_pretty(data)?)
        .with_context(|| format!("writing {}", tmp.display()))?;
    fs::rename(&tmp, path).with_context(|| format!("replacing {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Cycle, PriceChange, Subscription};
    use chrono::NaiveDate;

    fn temp_path(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("grana-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir.join("nested/grana.json")
    }

    #[test]
    fn missing_file_loads_empty() {
        let path = temp_path("missing");
        assert_eq!(load(&path).unwrap(), Data::default());
    }

    #[test]
    fn save_then_load_round_trips() {
        let path = temp_path("roundtrip");
        let mut data = Data::default();
        data.subscriptions.push(Subscription {
            name: "netflix".into(),
            cycle: Cycle::Monthly,
            prices: vec![PriceChange {
                cents: 4490,
                date: NaiveDate::from_ymd_opt(2026, 10, 30).unwrap(),
            }],
            archived: false,
        });
        save(&path, &data).unwrap();
        assert_eq!(load(&path).unwrap(), data);
    }
}
