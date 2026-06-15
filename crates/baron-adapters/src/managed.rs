use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

const START: &str = "<!-- BARON:MANAGED:START -->";
const END: &str = "<!-- BARON:MANAGED:END -->";
const ROUTING_START: &str = "<!-- BARON:ROUTING:START -->";
const ROUTING_END: &str = "<!-- BARON:ROUTING:END -->";

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

pub fn upsert_routing_block(
    path: &Path,
    managed_body: &str,
    custom_heading: &str,
    custom_guidance: &str,
) -> Result<()> {
    let block = format!("{ROUTING_START}\n{}\n{ROUTING_END}", managed_body.trim());
    let existing = fs::read_to_string(path).unwrap_or_default();
    let preserved = match (existing.find(ROUTING_START), existing.find(ROUTING_END)) {
        (Some(start), Some(end)) if end >= start => {
            let end = end + ROUTING_END.len();
            format!("{}{}", &existing[..start], &existing[end..])
        }
        _ => existing
            .find(custom_heading)
            .map(|index| existing[index..].to_string())
            .unwrap_or(existing),
    };
    let preserved = if preserved.trim().is_empty() {
        format!("{custom_heading}\n\n{custom_guidance}")
    } else {
        preserved.trim().to_string()
    };
    atomic_write(path, &format!("{block}\n\n{preserved}\n"))
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
