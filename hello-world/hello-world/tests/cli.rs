use assert_cmd::Command;

#[test]
fn hello_world_output() {
    let mut cmd = Command::cargo_bin("hello-world").unwrap();
    cmd.assert()
        .success()
        .stdout("Hello, World!\n");
}
