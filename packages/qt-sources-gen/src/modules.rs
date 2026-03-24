use std::collections::{BTreeSet, HashMap};

use crate::sources::SourcesFile;

pub const BASE_URL: &str = "https://download.qt.io/official_releases/qt";

/// Extract sorted archive filenames from md5sums.txt entries.
///
/// Filters to files matching the `*-everywhere-src-*.tar.xz` pattern,
/// excluding the `qt-everywhere-src` mega-bundle.
#[must_use]
pub fn archive_filenames<S: ::std::hash::BuildHasher>(md5_entries: &HashMap<String, String, S>) -> Vec<String> {
    let mut filenames: Vec<String> = md5_entries
        .keys()
        .filter(|f| f.ends_with(".tar.xz") && f.contains("-everywhere-src-") && !f.starts_with("qt-everywhere-src-"))
        .cloned()
        .collect();
    filenames.sort();
    filenames
}

/// Compare the set of modules in the existing sources file against what
/// md5sums.txt lists. Returns `true` if they match exactly.
#[must_use]
pub fn modules_match<S: ::std::hash::BuildHasher>(expected_md5s: &HashMap<String, String, S>, existing: &SourcesFile) -> bool {
    let upstream_vec = archive_filenames(expected_md5s);
    let upstream: BTreeSet<&str> = upstream_vec.iter().map(String::as_str).collect();
    let stored: BTreeSet<&str> = existing.sha512s.keys().map(String::as_str).collect();
    upstream == stored
}

/// Extract the module name from an archive filename.
///
/// e.g. `"qtbase-everywhere-src-6.11.0.tar.xz"` → `"qtbase"`
#[must_use]
pub fn module_name(filename: &str) -> &str {
    filename.split("-everywhere-src-").next().unwrap_or(filename)
}

/// Split "X.Y.Z" into ("X.Y", "Z"). Returns `None` if the format is invalid.
#[must_use]
pub fn split_version(version: &str) -> Option<(&str, &str)> {
    version.rsplit_once('.')
}

#[must_use]
pub fn archive_filename(module: &str, version: &str) -> String {
    format!("{module}-everywhere-src-{version}.tar.xz")
}

#[must_use]
pub fn archive_url(major_minor: &str, version: &str, filename: &str) -> String {
    format!("{BASE_URL}/{major_minor}/{version}/submodules/{filename}")
}

#[must_use]
pub fn md5sums_url(major_minor: &str, version: &str) -> String {
    format!("{BASE_URL}/{major_minor}/{version}/submodules/md5sums.txt")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_version_valid() {
        assert_eq!(split_version("6.11.0"), Some(("6.11", "0")));
    }

    #[test]
    fn split_version_no_dot() {
        assert_eq!(split_version("611"), None);
    }

    #[test]
    fn archive_filename_format() {
        assert_eq!(archive_filename("qtbase", "6.11.0"), "qtbase-everywhere-src-6.11.0.tar.xz");
    }

    #[test]
    fn archive_url_format() {
        let url = archive_url("6.11", "6.11.0", "qtbase-everywhere-src-6.11.0.tar.xz");
        assert_eq!(
            url,
            "https://download.qt.io/official_releases/qt/6.11/6.11.0/submodules/qtbase-everywhere-src-6.11.0.tar.xz"
        );
    }

    #[test]
    fn md5sums_url_format() {
        assert_eq!(
            md5sums_url("6.11", "6.11.0"),
            "https://download.qt.io/official_releases/qt/6.11/6.11.0/submodules/md5sums.txt"
        );
    }

    #[test]
    fn archive_filenames_filters_and_sorts() {
        let mut entries = HashMap::new();
        entries.insert("qtsvg-everywhere-src-6.11.0.tar.xz".into(), "aaa".into());
        entries.insert("qtbase-everywhere-src-6.11.0.tar.xz".into(), "bbb".into());
        // mega-bundle should be excluded
        entries.insert("qt-everywhere-src-6.11.0.tar.xz".into(), "ccc".into());
        // non-archive should be excluded
        entries.insert("sha256sums.txt".into(), "ddd".into());

        let result = archive_filenames(&entries);
        assert_eq!(
            result,
            vec!["qtbase-everywhere-src-6.11.0.tar.xz", "qtsvg-everywhere-src-6.11.0.tar.xz",]
        );
    }

    #[test]
    fn module_name_extracts_prefix() {
        assert_eq!(module_name("qtbase-everywhere-src-6.11.0.tar.xz"), "qtbase");
        assert_eq!(module_name("qt5compat-everywhere-src-6.11.0.tar.xz"), "qt5compat");
    }

    #[test]
    fn module_name_fallback() {
        assert_eq!(module_name("unknown-file.tar.xz"), "unknown-file.tar.xz");
    }

    #[test]
    fn modules_match_identical() {
        let mut md5s = HashMap::new();
        md5s.insert("qtbase-everywhere-src-6.11.0.tar.xz".into(), "aaa".into());
        md5s.insert("qtsvg-everywhere-src-6.11.0.tar.xz".into(), "bbb".into());

        let mut sha512s = HashMap::new();
        sha512s.insert("qtbase-everywhere-src-6.11.0.tar.xz".into(), "hash1".into());
        sha512s.insert("qtsvg-everywhere-src-6.11.0.tar.xz".into(), "hash2".into());

        let existing = SourcesFile {
            sentinel: Some("sentinel".into()),
            sha512s,
        };
        assert!(modules_match(&md5s, &existing));
    }

    #[test]
    fn modules_match_extra_upstream() {
        let mut md5s = HashMap::new();
        md5s.insert("qtbase-everywhere-src-6.11.0.tar.xz".into(), "aaa".into());
        md5s.insert("qtsvg-everywhere-src-6.11.0.tar.xz".into(), "bbb".into());
        md5s.insert("qtnew-everywhere-src-6.11.0.tar.xz".into(), "ccc".into());

        let mut sha512s = HashMap::new();
        sha512s.insert("qtbase-everywhere-src-6.11.0.tar.xz".into(), "hash1".into());
        sha512s.insert("qtsvg-everywhere-src-6.11.0.tar.xz".into(), "hash2".into());

        let existing = SourcesFile {
            sentinel: Some("sentinel".into()),
            sha512s,
        };
        assert!(!modules_match(&md5s, &existing));
    }

    #[test]
    fn modules_match_extra_stored() {
        let mut md5s = HashMap::new();
        md5s.insert("qtbase-everywhere-src-6.11.0.tar.xz".into(), "aaa".into());

        let mut sha512s = HashMap::new();
        sha512s.insert("qtbase-everywhere-src-6.11.0.tar.xz".into(), "hash1".into());
        sha512s.insert("qtsvg-everywhere-src-6.11.0.tar.xz".into(), "hash2".into());

        let existing = SourcesFile {
            sentinel: Some("sentinel".into()),
            sha512s,
        };
        assert!(!modules_match(&md5s, &existing));
    }

    #[test]
    fn modules_match_ignores_mega_bundle() {
        let mut md5s = HashMap::new();
        md5s.insert("qtbase-everywhere-src-6.11.0.tar.xz".into(), "aaa".into());
        // mega-bundle in md5sums but not in sources — should still match
        md5s.insert("qt-everywhere-src-6.11.0.tar.xz".into(), "bbb".into());

        let mut sha512s = HashMap::new();
        sha512s.insert("qtbase-everywhere-src-6.11.0.tar.xz".into(), "hash1".into());

        let existing = SourcesFile {
            sentinel: Some("sentinel".into()),
            sha512s,
        };
        assert!(modules_match(&md5s, &existing));
    }
}
