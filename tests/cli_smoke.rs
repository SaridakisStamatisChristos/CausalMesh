use std::process::Command;
use tempfile::TempDir;

#[test]
fn documented_cli_happy_path_works() {
    let dir_a = TempDir::new().unwrap();
    let dir_b = TempDir::new().unwrap();
    let bin = env!("CARGO_BIN_EXE_causalmesh");

    let run = |args: &[&str]| {
        let output = Command::new(bin).args(args).output().unwrap();
        assert!(
            output.status.success(),
            "command {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
        output
    };

    run(&["init", dir_a.path().to_str().unwrap()]);
    run(&["init", dir_b.path().to_str().unwrap()]);
    run(&[
        "put",
        dir_a.path().to_str().unwrap(),
        "hello",
        "world",
    ]);
    run(&[
        "sync",
        dir_a.path().to_str().unwrap(),
        dir_b.path().to_str().unwrap(),
    ]);
    let output = run(&["get", dir_b.path().to_str().unwrap(), "hello"]);
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "world");
    run(&["verify", dir_a.path().to_str().unwrap()]);
    run(&["export-summary", dir_b.path().to_str().unwrap()]);
}
