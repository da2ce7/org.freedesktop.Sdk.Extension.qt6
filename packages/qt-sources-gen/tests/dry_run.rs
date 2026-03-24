use qt_sources_gen::sentinel::{self, SentinelStatus};
use qt_sources_gen::{client, sources};

/// Dry-run against a real Qt version with an empty sources file.
/// This only fetches md5sums.txt (small file) and verifies the sentinel logic works.
#[test]
fn dry_run_detects_missing_sentinel() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init()
        .ok();

    let dir = tempfile::tempdir().unwrap();
    let sources_path = dir.path().join("sources");
    std::fs::write(&sources_path, "").unwrap();

    let http = client::build_client().unwrap();
    let current = sentinel::fetch_sentinel(&http, "6.11.0").unwrap();
    let existing = sources::read_sources(sources_path.to_str().unwrap()).unwrap();
    let status = sentinel::compare_sentinel(&current, &existing);

    assert_eq!(status, SentinelStatus::Missing);
    assert!(!current.hash.is_empty(), "sentinel hash should not be empty");
}

/// Dry-run with a matching sentinel should detect unchanged status.
#[test]
fn dry_run_up_to_date_when_sentinel_matches() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init()
        .ok();

    let dir = tempfile::tempdir().unwrap();
    let sources_path = dir.path().join("sources");

    // First, get the current sentinel
    let http = client::build_client().unwrap();
    let current = sentinel::fetch_sentinel(&http, "6.11.0").unwrap();

    // Write the sentinel to the sources file
    let sentinel_line = format!("SHA512 (md5sums.txt) = {}", current.hash);
    std::fs::write(&sources_path, &sentinel_line).unwrap();

    // Now check again — should be unchanged
    let existing = sources::read_sources(sources_path.to_str().unwrap()).unwrap();
    let status = sentinel::compare_sentinel(&current, &existing);
    assert_eq!(status, SentinelStatus::Unchanged);
}

/// The workspace `sources` file must contain a valid sentinel line.
/// This test fails if the file is empty or the sentinel is missing,
/// ensuring CI catches an unpopulated sources file.
#[test]
fn sources_file_has_sentinel() {
    let sources_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../sources");
    let existing =
        sources::read_sources(sources_path).unwrap_or_else(|e| panic!("failed to read sources file at {sources_path}: {e}"));

    assert!(
        existing.sentinel.is_some(),
        "sources file is missing the sentinel line — run qt-sources-gen to populate it"
    );
}
