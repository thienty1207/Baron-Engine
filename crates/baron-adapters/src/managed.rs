use std::path::Path;

use anyhow::{bail, Result};
use baron_core::safe_io::{read_text, replace_text};

const START: &str = "<!-- BARON:MANAGED:START -->";
const END: &str = "<!-- BARON:MANAGED:END -->";
const ROUTING_START: &str = "<!-- BARON:ROUTING:START -->";
const ROUTING_END: &str = "<!-- BARON:ROUTING:END -->";

pub fn upsert_managed_block(path: &Path, body: &str) -> Result<()> {
    let block = format!("{START}\n{}\n{END}", body.trim());
    let existing = read_text(path)?.unwrap_or_default();
    let updated = match delimited_block_bounds(&existing, START, END, "managed")? {
        Some((start, end)) => {
            let end = end + END.len();
            format!("{}{}{}", &existing[..start], block, &existing[end..])
        }
        None if existing.trim().is_empty() => format!("{block}\n"),
        None => format!("{}\n\n{block}\n", existing.trim_end()),
    };
    replace_text(path, &updated)
}

pub fn write_managed_file(path: &Path, content: &str) -> Result<()> {
    replace_text(path, content)
}

pub fn upsert_routing_block(
    path: &Path,
    managed_body: &str,
    custom_heading: &str,
    custom_guidance: &str,
) -> Result<()> {
    let block = format!("{ROUTING_START}\n{}\n{ROUTING_END}", managed_body.trim());
    let existing = read_text(path)?.unwrap_or_default();
    let preserved = match delimited_block_bounds(&existing, ROUTING_START, ROUTING_END, "routing")?
    {
        Some((start, end)) => {
            let end = end + ROUTING_END.len();
            format!("{}{}", &existing[..start], &existing[end..])
        }
        None => existing
            .find(custom_heading)
            .map(|index| existing[index..].to_string())
            .unwrap_or(existing),
    };
    let preserved = if preserved.trim().is_empty() {
        format!("{custom_heading}\n\n{custom_guidance}")
    } else {
        preserved.trim().to_string()
    };
    replace_text(path, &format!("{block}\n\n{preserved}\n"))
}

pub(crate) fn delimited_block_bounds(
    content: &str,
    start: &str,
    end: &str,
    label: &str,
) -> Result<Option<(usize, usize)>> {
    let start_count = content.match_indices(start).count();
    let end_count = content.match_indices(end).count();
    match (start_count, end_count) {
        (0, 0) => Ok(None),
        (1, 1) => {
            let start_index = content.find(start).expect("counted managed start marker");
            let end_index = content.find(end).expect("counted managed end marker");
            if end_index < start_index {
                bail!("Baron {label} markers are malformed: end marker appears before start marker");
            }
            Ok(Some((start_index, end_index)))
        }
        _ => bail!(
            "Baron {label} markers are malformed: expected zero or one complete marker pair, found {start_count} start marker(s) and {end_count} end marker(s)"
        ),
    }
}
