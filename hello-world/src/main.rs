fn main() {
    println!("Hello World");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn test_hello_world_output() {
        let output = Command::new("cargo")
            .arg("run")
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "Hello World");
    }
}
