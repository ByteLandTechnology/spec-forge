use assert_cmd::Command;

#[test]
fn top_level_help_smoke_test() {
    Command::cargo_bin("spec-forge-cli")
        .expect("binary should build")
        .arg("--help")
        .assert()
        .success();
}
