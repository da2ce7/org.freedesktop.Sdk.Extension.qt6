use qt_sources_gen::{CheckResult, check_sources};

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

    let result = check_sources("6.11.0", sources_path.to_str().unwrap(), true);

    match &result {
        CheckResult::Changed { new_sources } => {
            assert!(
                new_sources.contains("SHA512 (md5sums.txt) ="),
                "sentinel line missing from output: {new_sources}"
            );
        }
        other => panic!("expected Changed, got {other:?}"),
    }
}

/// Dry-run with a matching sentinel should return `UpToDate`.
#[test]
fn dry_run_up_to_date_when_sentinel_matches() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init()
        .ok();

    // First, get the current sentinel
    let dir = tempfile::tempdir().unwrap();
    let sources_path = dir.path().join("sources");
    std::fs::write(&sources_path, "").unwrap();

    let result = check_sources("6.11.0", sources_path.to_str().unwrap(), true);
    let sentinel = match result {
        CheckResult::Changed { new_sources } => new_sources,
        other => panic!("expected Changed for initial run, got {other:?}"),
    };

    // Write the sentinel to the sources file
    std::fs::write(&sources_path, &sentinel).unwrap();

    // Now check again — should be up to date
    let result = check_sources("6.11.0", sources_path.to_str().unwrap(), true);
    assert_eq!(result, CheckResult::UpToDate);
}

/// The workspace `sources` file must contain a valid sentinel line.
/// This test fails if the file is empty or the sentinel is missing,
/// ensuring CI catches an unpopulated sources file.
#[test]
fn sources_file_has_sentinel() {
    let sources_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../sources");
    let content =
        std::fs::read_to_string(sources_path).unwrap_or_else(|e| panic!("failed to read sources file at {sources_path}: {e}"));

    assert!(
        content.lines().any(|l| l.starts_with("SHA512 (md5sums.txt) =")),
        "sources file is missing the sentinel line — run qt-sources-gen to populate it"
    );
}
