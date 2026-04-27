use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

fn main() {
    let mut cmd = Command::new("zpool");
    cmd.args(["events", "-vHf"])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    cmd.exec();
}
