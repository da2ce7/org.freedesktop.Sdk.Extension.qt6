use clap::Parser;
use qt_sources_gen::{CheckResult, check_sources};
use tracing::info;

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
}

fn main() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".parse().unwrap()))
        .with_writer(std::io::stderr)
        .init();

    let args = Args::parse();
    info!(version = %args.version, sources = %args.sources, dry_run = args.dry_run, "starting");

    let result = check_sources(&args.version, &args.sources, args.dry_run);
    let exit_code = match &result {
        CheckResult::UpToDate => 0,
        CheckResult::Changed { .. } => 1,
        CheckResult::Error(_) => 2,
    };

    println!("{}", serde_json::to_string(&result).expect("failed to serialize result"));
    std::process::exit(exit_code);
}
