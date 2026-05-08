use std::{path::PathBuf, process, sync::Arc};

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use scanner_analysis::{Analyzer, NodeJsAnalyzer, Verdict, VsCodeAnalyzer};
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

    /// GitHub personal access token (overrides `GITHUB_TOKEN` env var)
    #[arg(long, value_name = "TOKEN")]
    token: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let exit_code = run(args).await?;
    process::exit(exit_code);
}

async fn run(args: Args) -> anyhow::Result<i32> {
    let token = resolve_token(args.token)?;
    let (owner, name) = parse_github_url(&args.github_url)?;

    let rules_path = args.rules.unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../rules/nodejs.toml")
    });
    let vscode_rules_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../rules/vscode.toml");
    let ruleset = Arc::new(load(&rules_path)?);
    let vscode_ruleset = Arc::new(load(&vscode_rules_path)?);

    let snapshot = fetch_snapshot(&args.github_url, &owner, &name, args.clone, &token).await?;

    let mut findings = NodeJsAnalyzer.analyze(&snapshot, &ruleset);
    findings.extend(VsCodeAnalyzer.analyze(&snapshot, &vscode_ruleset));
    let verdict = Verdict::from_findings(&findings);

    print!("{}", render(&findings, &verdict, true));

    Ok(match verdict {
        Verdict::Safe => 0,
        Verdict::Suspicious => 1,
        Verdict::Dangerous => 2,
    })
}

fn resolve_token(flag: Option<String>) -> anyhow::Result<String> {
    let token = if let Some(t) = flag {
        t
    } else if let Ok(t) = std::env::var("GITHUB_TOKEN") {
        if t.is_empty() {
            anyhow::bail!("GitHub token is required. Set GITHUB_TOKEN or pass --token <TOKEN>.")
        }
        t
    } else {
        anyhow::bail!("GitHub token is required. Set GITHUB_TOKEN or pass --token <TOKEN>.")
    };
    if token.bytes().any(|b| b < 32 || b == 127) {
        anyhow::bail!("GitHub token contains invalid characters");
    }
    Ok(token)
}

async fn fetch_snapshot(
    url: &str,
    owner: &str,
    name: &str,
    clone: bool,
    token: &str,
) -> anyhow::Result<scanner_repository::RepoSnapshot> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner} {msg}")
            .unwrap(),
    );
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    let snapshot = if clone {
        pb.set_message(format!("Cloning {owner}/{name}..."));
        LocalCloneClient::new(token).fetch_url(url)?
    } else {
        pb.set_message(format!("Fetching {owner}/{name}..."));
        GithubApiClient::new("https://api.github.com", token)
            .fetch(owner, name)
            .await?
    };
    pb.finish_and_clear();
    Ok(snapshot)
}

fn parse_github_url(url: &str) -> anyhow::Result<(String, String)> {
    let url = url.trim_end_matches('/');
    let parts: Vec<&str> = url.rsplitn(3, '/').collect();
    if parts.len() < 3 || !parts[2].contains("github.com") {
        anyhow::bail!("invalid GitHub URL — expected https://github.com/owner/repo");
    }
    Ok((parts[1].to_owned(), parts[0].to_owned()))
}
