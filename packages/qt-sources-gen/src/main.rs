use clap::Parser;
use qt_sources_gen::sentinel::{self, SentinelStatus};
use qt_sources_gen::sources::{self, SourcesFile};
use qt_sources_gen::{CheckResult, client, modules, verify};
use tracing::{info, warn};

#[derive(Parser)]
#[command(about = "Check Qt md5sums sentinel and regenerate SHA512 sources if changed")]
struct Args {
    /// Qt version (e.g. 6.11.0)
    #[arg(short, long)]
    version: String,

    /// Path to the cached sources file (contains the md5sums sentinel + archive hashes)
    #[arg(short, long, default_value = "sources")]
    sources: String,

    /// Dry run: only check the md5sums sentinel, skip downloading archives
    #[arg(long)]
    dry_run: bool,

    /// Force: download and verify all archives even if the md5sums sentinel is unchanged
    #[arg(long)]
    force: bool,
}

fn main() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".parse().unwrap()))
        .with_writer(std::io::stderr)
        .init();

    let args = Args::parse();
    info!(version = %args.version, sources = %args.sources, dry_run = args.dry_run, force = args.force, "starting");

    let result = run(&args);
    let exit_code = match &result {
        CheckResult::UpToDate => 0,
        CheckResult::Changed { .. } => 1,
        CheckResult::Error(_) => 2,
    };

    println!("{}", serde_json::to_string(&result).expect("failed to serialize result"));
    std::process::exit(exit_code);
}

fn run(args: &Args) -> CheckResult {
    let http = match client::build_client() {
        Ok(c) => c,
        Err(e) => return CheckResult::Error(format!("failed to build HTTP client: {e}")),
    };

    // Fetch and hash md5sums.txt
    let current = match sentinel::fetch_sentinel(&http, &args.version) {
        Ok(s) => s,
        Err(e) => return CheckResult::Error(format!("failed to fetch md5sums.txt: {e}")),
    };

    info!(sentinel = %current.hash, "computed sentinel");

    // Read existing sources file
    let existing = match sources::read_sources(&args.sources) {
        Ok(f) => f,
        Err(e) => return CheckResult::Error(format!("failed to read sources: {e}")),
    };

    // Compare sentinels
    let status = sentinel::compare_sentinel(&current, &existing);
    let sentinel_changed = match status {
        SentinelStatus::Unchanged => {
            info!("sentinel unchanged");
            false
        }
        SentinelStatus::Changed => {
            warn!("sentinel mismatch");
            true
        }
        SentinelStatus::Missing => {
            info!("no sentinel found — generating fresh");
            true
        }
    };

    // Check whether the module set in the sources file matches md5sums.txt
    let modules_changed = if sentinel_changed {
        // Already going to regenerate, no need to check
        false
    } else if !modules::modules_match(&current.expected_md5s, &existing) {
        warn!("module set mismatch between md5sums.txt and sources file");
        true
    } else {
        false
    };

    let needs_update = sentinel_changed || modules_changed;

    // Quick exit: nothing changed, not forcing
    if !needs_update && !args.force {
        info!("sources are up to date");
        return CheckResult::UpToDate;
    }

    if args.dry_run {
        info!("dry run — skipping archive downloads");
        return CheckResult::Changed {
            new_sources: format!("SHA512 (md5sums.txt) = {}", current.hash),
        };
    }

    // Determine whether we're verifying existing hashes or generating new ones
    let verify_only = !needs_update && args.force;
    let stored_sha512s = if verify_only {
        info!("force verify — downloading archives to verify MD5 + SHA-512 hashes");
        Some(&existing.sha512s)
    } else {
        info!("recomputing SHA-512 for all modules");
        None
    };

    let module_hashes = match verify::download_and_verify(&http, &args.version, &current.expected_md5s, stored_sha512s) {
        Ok(h) => h,
        Err(e) => return CheckResult::Error(e.to_string()),
    };

    if verify_only {
        info!("all hashes verified — sources are up to date");
        return CheckResult::UpToDate;
    }

    let entries: Vec<(String, String)> = module_hashes.into_iter().map(|m| (m.filename, m.sha512)).collect();

    CheckResult::Changed {
        new_sources: SourcesFile::serialize_ordered(&current.hash, &entries),
    }
}
