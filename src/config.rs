use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

pub fn rime_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".local/share/fcitx5/rime")
}

pub fn dict_path() -> PathBuf {
    rime_dir().join("custom_words.dict.yaml")
}

pub fn config_path() -> PathBuf {
    rime_dir().join("rime_ice.custom.yaml")
}

pub fn confirm(prompt: &str) -> anyhow::Result<bool> {
    print!("{} (y/n) ", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_lowercase() == "y")
}

pub fn init_custom_words() -> anyhow::Result<()> {
    let path = dict_path();
    if path.exists() {
        if !confirm("custom_words.dict.yaml 已存在，是否覆盖？")? {
            println!("跳过创建词库文件");
            return Ok(());
        }
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let template = concat!(
        "---\n",
        "name: custom_words\n",
        "version: \"1.0\"\n",
        "sort: by_weight\n",
        "...\n",
        "#请不要轻易修改此文件，使用rime-custom-config修改\n",
    );
    fs::write(&path, template)?;
    println!("已创建 {}", path.display());
    Ok(())
}

pub fn init_config() -> anyhow::Result<()> {
    let path = config_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    if !path.exists() {
        let content = "patch:\n  translator/dictionary: custom_words\n";
        fs::write(&path, content)?;
        println!("已创建 {}", path.display());
        return Ok(());
    }

    let content = fs::read_to_string(&path)?;
    let lines: Vec<&str> = content.lines().collect();

    let td_idx = lines.iter().position(|l| {
        let trimmed = l.trim_start();
        trimmed.starts_with("translator/dictionary:")
    });

    if let Some(idx) = td_idx {
        let line = lines[idx];
        let value = line.split_once(':').map(|(_, v)| v.trim()).unwrap_or("");
        if value == "custom_words" {
            println!("translator/dictionary 已配置为 custom_words，跳过");
            return Ok(());
        }
        if !confirm(&format!(
            "已存在 translator/dictionary = {}，是否覆盖为 custom_words？",
            value
        ))? {
            anyhow::bail!("用户取消操作");
        }
    }

    let mut new_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
    if let Some(idx) = td_idx {
        new_lines.remove(idx);
    }

    if let Some(patch_idx) = new_lines.iter().position(|l| l.trim() == "patch:") {
        let patch_indent = new_lines[patch_idx]
            .chars()
            .take_while(|c| c.is_whitespace())
            .count();
        let child_indent = " ".repeat(patch_indent + 2);

        let mut insert_idx = patch_idx;
        for i in (patch_idx + 1)..new_lines.len() {
            if new_lines[i].is_empty() {
                continue;
            }
            let line_indent = new_lines[i].chars().take_while(|c| c.is_whitespace()).count();
            if line_indent > patch_indent {
                insert_idx = i;
            } else {
                break;
            }
        }

        new_lines.insert(
            insert_idx + 1,
            format!("{}translator/dictionary: custom_words", child_indent),
        );
        fs::write(&path, new_lines.join("\n") + "\n")?;
        println!("已添加 translator/dictionary 到 patch");
    } else {
        let new_content = format!(
            "{}\npatch:\n  translator/dictionary: custom_words\n",
            content.trim_end()
        );
        fs::write(&path, new_content)?;
        println!("已添加 patch 配置");
    }

    Ok(())
}
