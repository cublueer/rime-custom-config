use std::process::{Command, Stdio};

use anyhow::Context;

pub fn deploy() -> anyhow::Result<()> {
    Command::new("fcitx5")
        .arg("-r")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("无法执行 fcitx5 -r，请确保 fcitx5 已安装且在 PATH 中")?;

    println!("fcitx5 已发送重启信号");
    Ok(())
}
