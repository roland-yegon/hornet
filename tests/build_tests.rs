use std::fs;
use std::process::Command;

#[test]
#[ignore = "requires LLVM and a system linker"]
fn build_produces_native_binary() {
    let source = "print(\"Built!\")\n";
    let tmp = "/tmp/hornet_build_test.hn";
    fs::write(tmp, source).expect("write source failed");

    let status = Command::new("cargo")
        .args(&["run", "--", "build", tmp])
        .status()
        .expect("failed to execute hornet build");
    assert!(status.success());

    let binary = "/tmp/hornet_build_test";
    assert!(fs::metadata(binary).is_ok());

    let output = Command::new(binary)
        .output()
        .expect("failed to execute built binary");
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "Built!");

    fs::remove_file(tmp).ok();
    fs::remove_file(binary).ok();
}
