use std::fs;
use std::path::Path;

use anyhow::Context;

pub struct WordEntry {
    pub word: String,
    pub pinyin: String,
    pub weight: u32,
}

impl WordEntry {
    pub fn new(word: String, pinyin: String, weight: u32) -> Self {
        Self {
            word,
            pinyin,
            weight,
        }
    }

    fn to_line(&self) -> String {
        format!("{}\t{}\t{}", self.word, self.pinyin, self.weight)
    }
}

pub struct DictFile {
    header: String,
    trailing: Vec<String>,
    entries: Vec<WordEntry>,
}

impl DictFile {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("无法读取文件 {}", path.display()))?;

        let lines: Vec<&str> = content.lines().collect();

        let header_start = lines
            .iter()
            .position(|l| l.trim() == "---")
            .ok_or_else(|| anyhow::anyhow!("词库文件格式错误：缺少 '---'"))?;

        let header_end = lines[header_start..]
            .iter()
            .position(|l| l.trim() == "...")
            .map(|p| header_start + p)
            .ok_or_else(|| anyhow::anyhow!("词库文件格式错误：缺少 '...'"))?;

        let header = lines[..=header_end].join("\n");

        let mut trailing = Vec::new();
        let mut entries = Vec::new();

        for line in &lines[header_end + 1..] {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                trailing.push(line.to_string());
                continue;
            }

            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 2 {
                let word = parts[0].trim().to_string();
                let pinyin = parts[1].trim().to_string();
                let weight = parts
                    .get(2)
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(0);
                if !word.is_empty() && !pinyin.is_empty() {
                    entries.push(WordEntry { word, pinyin, weight });
                    continue;
                }
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                entries.push(WordEntry {
                    word: parts[0].to_string(),
                    pinyin: parts[1].to_string(),
                    weight: parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0),
                });
            } else {
                trailing.push(line.to_string());
            }
        }

        Ok(Self {
            header,
            trailing,
            entries,
        })
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let mut content = self.header.clone();
        content.push('\n');
        for line in &self.trailing {
            content.push_str(line);
            content.push('\n');
        }
        for entry in &self.entries {
            content.push_str(&entry.to_line());
            content.push('\n');
        }
        fs::write(path, &content)
            .with_context(|| format!("无法写入文件 {}", path.display()))?;
        Ok(())
    }

    pub fn add(&mut self, entry: WordEntry) -> bool {
        if self.entries.iter().any(|e| e.word == entry.word) {
            return false;
        }
        self.entries.push(entry);
        true
    }

    pub fn remove(&mut self, word: &str) -> usize {
        let before = self.entries.len();
        self.entries.retain(|e| e.word != word);
        before - self.entries.len()
    }

    pub fn search(&self, keyword: &str) -> Vec<&WordEntry> {
        let kw = keyword.to_lowercase();
        self.entries
            .iter()
            .filter(|e| e.word.contains(keyword) || e.pinyin.to_lowercase().contains(&kw))
            .collect()
    }

    pub fn list(&self) -> &[WordEntry] {
        &self.entries
    }
}
