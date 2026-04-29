mod error;

use std::{path::PathBuf, process, sync::Arc};

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use scanner_analysis::{Analyzer, NodeJsAnalyzer, Verdict};
use scanner_report::render;
use scanner_repository::{GithubApiClient, LocalCloneClient, RepositoryPort};
use scanner_rules::loader::load;

#[derive(Parser)]
#[command(
    name = "repo-scanner",
    about = "Scan a GitHub repo for malware patterns before cloning"
)]
struct Args {
    /// GitHub repository URL (e.g. `https://github.com/owner/repo`)
    github_url: String,

    /// Clone repo locally for deeper analysis (default: API only)
    #[arg(long)]
    clone: bool,

    /// Path to custom TOML ruleset
    #[arg(long, value_name = "FILE")]
    rules: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let (owner, name) = parse_github_url(&args.github_url)?;

    let rules_path = args.rules.unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../rules/nodejs.toml")
    });
    let ruleset = Arc::new(load(&rules_path)?);

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner} {msg}")
            .unwrap(),
    );
    pb.set_message(format!("Fetching {owner}/{name}..."));
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    let snapshot = if args.clone {
        pb.set_message(format!("Cloning {owner}/{name}..."));
        LocalCloneClient::new().fetch_url(&args.github_url)?
    } else {
        GithubApiClient::new("https://api.github.com")
            .fetch(&owner, &name)
            .await?
    };
    pb.finish_and_clear();

    let findings = NodeJsAnalyzer.analyze(&snapshot, &ruleset);
    let verdict = Verdict::from_findings(&findings);

    let output = render(&findings, &verdict, true);
    print!("{output}");

    let exit_code = match verdict {
        Verdict::Safe => 0,
        Verdict::Suspicious => 1,
        Verdict::Dangerous => 2,
    };
    process::exit(exit_code);
}

fn parse_github_url(url: &str) -> anyhow::Result<(String, String)> {
    let url = url.trim_end_matches('/');
    let parts: Vec<&str> = url.rsplitn(3, '/').collect();
    if parts.len() < 2 {
        anyhow::bail!("invalid GitHub URL — expected https://github.com/owner/repo");
    }
    Ok((parts[1].to_owned(), parts[0].to_owned()))
}
