use std::fmt::Write as _;

use colored::Colorize;
use scanner_analysis::{Finding, Verdict};
use scanner_rules::Severity;

#[must_use]
pub fn render(findings: &[Finding], verdict: &Verdict, color: bool) -> String {
    if !color {
        colored::control::set_override(false);
    }
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
            let _ = write!(out, "\n  {label}  {loc}\n");
            let _ = writeln!(out, "           {}", f.message);
            if let Some(ref snippet) = f.snippet {
                let _ = writeln!(out, "           > {}", snippet.dimmed());
            }
        }
    }

    let banner = match verdict {
        Verdict::Safe => format!("\n  {}\n", " SAFE — no findings ".on_green().bold()),
        Verdict::Suspicious => format!(
            "\n  {} {} finding(s)\n",
            " SUSPICIOUS ".on_bright_yellow().black().bold(),
            findings.len()
        ),
        Verdict::Dangerous => format!(
            "\n  {} {} finding(s)\n",
            " DANGEROUS ".on_red().bold(),
            findings.len()
        ),
    };
    out.push_str(&banner);
    out
}
