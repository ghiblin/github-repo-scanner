use std::fmt::Write as _;

use colored::Colorize;
use scanner_analysis::{Finding, Verdict};
use scanner_rules::Severity;

#[must_use]
pub fn render(findings: &[Finding], verdict: &Verdict, color: bool) -> String {
    colored::control::set_override(color);
    let mut out = render_findings(findings);
    out.push_str(&render_banner(verdict, findings.len()));
    colored::control::unset_override();
    out
}

fn render_findings(findings: &[Finding]) -> String {
    let mut out = String::new();
    for severity in [
        Severity::Critical,
        Severity::High,
        Severity::Medium,
        Severity::Low,
    ] {
        let group: Vec<_> = findings.iter().filter(|f| f.severity == severity).collect();
        if group.is_empty() {
            continue;
        }
        for f in group {
            out.push_str(&render_finding(f, &severity));
        }
    }
    out
}

fn render_finding(f: &Finding, severity: &Severity) -> String {
    let label = match severity {
        Severity::Critical => " CRITICAL ".on_red().bold().to_string(),
        Severity::High => " HIGH ".on_yellow().black().to_string(),
        Severity::Medium => " MEDIUM ".on_bright_yellow().black().to_string(),
        Severity::Low => " LOW ".on_blue().to_string(),
    };
    let loc = match f.line {
        Some(l) if l > 0 => format!("{}:{l}", f.file.display()),
        _ => f.file.display().to_string(),
    };
    let mut out = format!("\n  {label}  {loc}\n           {}\n", f.message);
    if let Some(ref snippet) = f.snippet {
        let _ = writeln!(out, "           > {}", snippet.dimmed());
    }
    out
}

fn render_banner(verdict: &Verdict, count: usize) -> String {
    match verdict {
        Verdict::Safe => format!("\n  {}\n", " SAFE — no findings ".on_green().bold()),
        Verdict::Suspicious => format!(
            "\n  {} {count} finding(s)\n",
            " SUSPICIOUS ".on_bright_yellow().black().bold(),
        ),
        Verdict::Dangerous => {
            format!("\n  {} {count} finding(s)\n", " DANGEROUS ".on_red().bold(),)
        }
    }
}
