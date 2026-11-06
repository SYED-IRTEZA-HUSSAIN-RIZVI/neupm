use anyhow::{Context, Result, bail};
use flate2::read::GzDecoder;
use std::collections::HashSet;
use std::io::Cursor;
use std::path::Component;
use tar::Archive;

/// Inspect a tarball to dynamically find the single top-level root prefix directory.
///
/// While most packages use `package/`, some (like `@types/node`) use `node/`.
pub fn find_archive_root(archive_bytes: &[u8]) -> Result<String> {
    let decoder = GzDecoder::new(Cursor::new(archive_bytes));
    let mut archive = Archive::new(decoder);

    let mut top_level_dirs: HashSet<String> = HashSet::new();

    for entry_result in archive
        .entries()
        .context("Failed to read archive entries")?
    {
        let entry = entry_result.context("Failed to read archive entry")?;
        let path = entry
            .path()
            .context("Failed to read entry path")?
            .to_path_buf();

        let first_component = path.components().next().and_then(|c| match c {
            Component::Normal(s) => s.to_str().map(String::from),
            _ => None,
        });

        if let Some(dir_name) = first_component {
            top_level_dirs.insert(dir_name);
        }
    }

    if top_level_dirs.is_empty() {
        bail!("Archive is empty: no entries found.");
    }

    if top_level_dirs.len() != 1 {
        bail!(
            "Invalid npm archive: expected exactly one top-level directory, found {}: {:?}",
            top_level_dirs.len(),
            top_level_dirs
        );
    }

    Ok(top_level_dirs.into_iter().next().unwrap())
}
