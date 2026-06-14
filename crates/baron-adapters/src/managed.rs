use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

const START: &str = "<!-- BARON:MANAGED:START -->";
const END: &str = "<!-- BARON:MANAGED:END -->";

pub fn upsert_managed_block(path: &Path, body: &str) -> Result<()> {
    let block = format!("{START}\n{}\n{END}", body.trim());
    let existing = fs::read_to_string(path).unwrap_or_default();
    let updated = match (existing.find(START), existing.find(END)) {
        (Some(start), Some(end)) if end >= start => {
            let end = end + END.len();
            format!("{}{}{}", &existing[..start], block, &existing[end..])
        }
        _ if existing.trim().is_empty() => format!("{block}\n"),
        _ => format!("{}\n\n{block}\n", existing.trim_end()),
    };
    atomic_write(path, &updated)
}

pub fn write_managed_file(path: &Path, content: &str) -> Result<()> {
    atomic_write(path, content)
}

fn atomic_write(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension("baron-tmp");
    fs::write(&temp, content).with_context(|| format!("Could not write {}", temp.display()))?;
    if path.exists() {
        fs::remove_file(path).with_context(|| format!("Could not replace {}", path.display()))?;
    }
    fs::rename(&temp, path).with_context(|| format!("Could not write {}", path.display()))
}
