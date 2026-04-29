use std::process::Command;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_repo-scanner"))
}

#[test]
fn shows_help_without_args() {
    let output = bin().arg("--help").output().expect("binary must exist");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("GITHUB_URL"));
}
