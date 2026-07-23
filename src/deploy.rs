use std::process::Command;

use anyhow::Context;

pub fn deploy() -> anyhow::Result<()> {
    let status = Command::new("fcitx5")
        .arg("-r")
        .status()
        .context("无法执行 fcitx5 -r，请确保 fcitx5 已安装且在 PATH 中")?;

    if !status.success() {
        anyhow::bail!("fcitx5 -r 执行失败");
    }

    println!("fcitx5 已重新部署");
    Ok(())
}
