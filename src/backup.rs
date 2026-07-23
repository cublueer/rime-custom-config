use std::path::PathBuf;
use std::fs;

use anyhow::Context;
use chrono::Local;

use crate::config;
use crate::dict::DictFile;

pub fn backup_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".cache/rime-custom-config")
}

pub fn sorted_backups() -> anyhow::Result<Vec<PathBuf>> {
    let dir = backup_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
        return Ok(Vec::new());
    }
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .collect();
    files.sort();
    Ok(files)
}

pub fn get_backup(index: usize) -> anyhow::Result<PathBuf> {
    let backups = sorted_backups()?;
    if index == 0 || index > backups.len() {
        anyhow::bail!("序号 {} 无效，共 {} 个备份", index, backups.len());
    }
    Ok(backups[index - 1].clone())
}

pub fn create_backup() -> anyhow::Result<()> {
    let dict_path = config::dict_path();
    if !dict_path.exists() {
        anyhow::bail!("词库文件不存在，请先运行 init");
    }

    let bdir = backup_dir();
    fs::create_dir_all(&bdir)?;

    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let name = format!("custom_words.{}.dict.yaml", timestamp);
    let backup = bdir.join(&name);

    fs::copy(&dict_path, &backup)
        .with_context(|| format!("备份失败: {}", backup.display()))?;

    println!("已备份到 {}", backup.display());
    Ok(())
}

pub fn list_backups() -> anyhow::Result<()> {
    let backups = sorted_backups()?;
    if backups.is_empty() {
        println!("暂无备份");
        return Ok(());
    }
    for (i, path) in backups.iter().enumerate() {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
        println!("{:>3}. {} ({} bytes)", i + 1, name, size);
    }
    Ok(())
}

fn print_entries(entries: &[crate::dict::WordEntry], line_numbers: bool) {
    if entries.is_empty() {
        println!("(空)");
        return;
    }
    for (i, entry) in entries.iter().enumerate() {
        if line_numbers {
            println!("{:>4}. {}\t{}\t{}", i + 1, entry.word, entry.pinyin, entry.weight);
        } else {
            println!("{}\t{}\t{}", entry.word, entry.pinyin, entry.weight);
        }
    }
}

pub fn show_backup(index: usize, line_numbers: bool) -> anyhow::Result<()> {
    let path = get_backup(index)?;
    let dict = DictFile::load(&path)?;
    print_entries(dict.list(), line_numbers);
    Ok(())
}

pub fn search_backup(index: usize, keyword: &str) -> anyhow::Result<()> {
    let path = get_backup(index)?;
    let dict = DictFile::load(&path)?;
    let results = dict.search(keyword);
    if results.is_empty() {
        println!("未找到匹配词条");
    } else {
        for entry in &results {
            println!("{}\t{}\t{}", entry.word, entry.pinyin, entry.weight);
        }
    }
    Ok(())
}

pub fn restore_backup(index: usize) -> anyhow::Result<()> {
    let backup_path = get_backup(index)?;
    let dict_path = config::dict_path();

    if !config::confirm("确认还原备份？")? {
        println!("已取消");
        return Ok(());
    }

    fs::copy(&backup_path, &dict_path).context("还原备份失败")?;
    println!("已还原备份 #{}", index);

    if config::confirm("是否立即部署？")? {
        crate::deploy::deploy()?;
    }
    Ok(())
}

pub fn delete_backup(index: Option<usize>) -> anyhow::Result<()> {
    match index {
        Some(idx) => {
            let path = get_backup(idx)?;
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if !config::confirm(&format!("确认删除备份 {}？", name))? {
                println!("已取消");
                return Ok(());
            }
            fs::remove_file(&path)?;
            println!("已删除备份 #{}", idx);
        }
        None => {
            let backups = sorted_backups()?;
            if backups.is_empty() {
                println!("暂无备份");
                return Ok(());
            }
            if !config::confirm(&format!("确认删除所有 {} 个备份？", backups.len()))? {
                println!("已取消");
                return Ok(());
            }
            for p in &backups {
                fs::remove_file(p)?;
            }
            println!("已删除所有备份");
        }
    }
    Ok(())
}
