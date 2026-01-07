use predicates::prelude::*;

#[test]
fn scc_snake_conversion() {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("dt");
    cmd.args(["scc", "-f", "snake", "HelloWorld"])
        .assert()
        .success()
        .stdout(predicate::str::contains("hello_world"));
}

#[test]
fn scc_list_formats() {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("dt");
    cmd.args(["scc", "--list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("camel"));
}

#[test]
fn ucc_quiet_base64_decode() {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("dt");
    cmd.args(["ucc", "-q", "aGVsbG8="])
        .assert()
        .success()
        .stdout(predicate::eq("hello"));
}

#[test]
fn ucc_text_lists_encodings() {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("dt");
    cmd.args(["ucc", "hello"]) // pure text => list encodings
        .assert()
        .success()
    .stdout(predicate::str::contains("\"subtitle\":\"Base64编码\""));
}

#[test]
fn ucc_json_decode_mode() {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("dt");
    cmd.args(["ucc", "--json", "aGVsbG8="])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"result\":\"hello\""));
}

#[test]
fn ts_parses_seconds_timestamp() {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("dt");
    cmd.args(["ts", "1700000000"])
        .assert()
        .success()
    .stdout(predicate::str::contains("\"subtitle\":\"Timestamp (Seconds)\""))
    .stdout(predicate::str::contains("\"arg\":\"1700000000\""));
}
