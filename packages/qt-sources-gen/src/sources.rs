use std::collections::HashMap;
use std::path::Path;

use crate::error::Error;

/// Parsed representation of the `sources` file.
#[derive(Debug, Default)]
pub struct SourcesFile {
    /// The sentinel line value: SHA-512 hash of md5sums.txt (without prefix).
    pub sentinel: Option<String>,
    /// Map of filename → SHA-512 hash for each archive.
    pub sha512s: HashMap<String, String>,
}

impl SourcesFile {
    /// Serialize back to the on-disk format.
    #[must_use]
    pub fn serialize(&self, sentinel_hash: &str) -> String {
        let mut lines = vec![format!("SHA512 (md5sums.txt) = {sentinel_hash}")];
        // We don't guarantee ordering here — callers supply ordered entries via `from_entries`.
        for (filename, hash) in &self.sha512s {
            lines.push(format!("SHA512 ({filename}) = {hash}"));
        }
        lines.join("\n")
    }

    /// Build a sources file from a sentinel hash and an ordered list of (filename, sha512) pairs.
    #[must_use]
    pub fn from_entries(sentinel_hash: &str, entries: &[(String, String)]) -> Self {
        let mut sha512s = HashMap::with_capacity(entries.len());
        for (filename, hash) in entries {
            sha512s.insert(filename.clone(), hash.clone());
        }
        Self {
            sentinel: Some(sentinel_hash.to_string()),
            sha512s,
        }
    }

    /// Serialize from a sentinel hash and ordered entries, preserving insertion order.
    #[must_use]
    pub fn serialize_ordered(sentinel_hash: &str, entries: &[(String, String)]) -> String {
        let mut lines = Vec::with_capacity(entries.len() + 1);
        lines.push(format!("SHA512 (md5sums.txt) = {sentinel_hash}"));
        for (filename, hash) in entries {
            lines.push(format!("SHA512 ({filename}) = {hash}"));
        }
        lines.join("\n")
    }
}

/// # Errors
/// Returns an error if the file cannot be read (missing files return a default).
pub fn read_sources(path: &str) -> Result<SourcesFile, Error> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(SourcesFile::default()),
        Err(e) => return Err(e.into()),
    };
    Ok(parse_sources(&content))
}

/// # Errors
/// Returns an error if the file or parent directories cannot be written.
pub fn write_sources(path: &str, content: &str) -> Result<(), Error> {
    if let Some(parent) = Path::new(path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)?;
    Ok(())
}

fn parse_sources(content: &str) -> SourcesFile {
    let mut file = SourcesFile::default();
    for line in content.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("SHA512 (")
            && let Some((name, hash)) = rest.split_once(") = ")
        {
            let hash = hash.trim().to_string();
            if name == "md5sums.txt" {
                file.sentinel = Some(hash);
            } else {
                file.sha512s.insert(name.to_string(), hash);
            }
        }
    }
    file
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty() {
        let f = parse_sources("");
        assert!(f.sentinel.is_none());
        assert!(f.sha512s.is_empty());
    }

    #[test]
    fn parse_full() {
        let content = "\
SHA512 (md5sums.txt) = abc123
SHA512 (qtbase-everywhere-src-6.11.0.tar.xz) = def456
SHA512 (qtsvg-everywhere-src-6.11.0.tar.xz) = 789abc";

        let f = parse_sources(content);
        assert_eq!(f.sentinel.as_deref(), Some("abc123"));
        assert_eq!(f.sha512s.len(), 2);
        assert_eq!(f.sha512s["qtbase-everywhere-src-6.11.0.tar.xz"], "def456");
        assert_eq!(f.sha512s["qtsvg-everywhere-src-6.11.0.tar.xz"], "789abc");
    }

    #[test]
    fn serialize_ordered_roundtrip() {
        let entries = vec![
            ("qtbase-everywhere-src-6.11.0.tar.xz".into(), "def456".into()),
            ("qtsvg-everywhere-src-6.11.0.tar.xz".into(), "789abc".into()),
        ];
        let text = SourcesFile::serialize_ordered("abc123", &entries);
        assert_eq!(
            text,
            "SHA512 (md5sums.txt) = abc123\n\
             SHA512 (qtbase-everywhere-src-6.11.0.tar.xz) = def456\n\
             SHA512 (qtsvg-everywhere-src-6.11.0.tar.xz) = 789abc"
        );
    }

    #[test]
    fn read_missing_file_returns_default() {
        let f = read_sources("/nonexistent/path/sources").unwrap();
        assert!(f.sentinel.is_none());
    }
}
