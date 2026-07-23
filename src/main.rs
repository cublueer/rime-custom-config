mod backup;
mod config;
mod deploy;
mod dict;

use anyhow::Context;
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rime-custom-config", about = "管理 fcitx5 rime 自定义中文词库")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// 初始化词库文件和 rime_ice.custom.yaml 配置
    Init,
    /// 追加词条（空格输入自动转为 Tab 写入，去重检查）
    Add {
        word: String,
        pinyin: String,
        #[arg(default_value = "0")]
        weight: u32,
    },
    /// 按词语删除词条
    Remove {
        word: String,
    },
    /// 列出所有自定义词条
    List {
        /// 按行号列出
        #[arg(short = 'l', long = "ln")]
        line_numbers: bool,
    },
    /// 模糊搜索词条（匹配词语或拼音）
    Search {
        keyword: String,
    },
    /// 清理 init 添加的配置和词库文件
    Cleanup,
    /// 重新部署 fcitx5 (fcitx5 -r)
    Deploy,
    /// 备份管理
    Backup(BackupArgs),
}

#[derive(Args)]
#[command(args_conflicts_with_subcommands = true)]
struct BackupArgs {
    /// 列出所有备份（带序号）
    #[arg(short = 'l', long = "list")]
    list_backups: bool,

    /// 还原指定序号的备份
    #[arg(short = 'r', long = "reset")]
    reset: Option<usize>,

    /// 删除备份（不带参数删除全部，带参数删除指定序号）
    #[arg(short = 'd', long = "delete", num_args = 0..=1, default_missing_value = "all")]
    delete: Option<String>,

    #[command(subcommand)]
    command: Option<BackupCommand>,
}

#[derive(Subcommand)]
enum BackupCommand {
    /// 查看备份中词条内容
    List {
        index: usize,
        /// 按行号列出
        #[arg(short = 'l', long = "ln")]
        line_numbers: bool,
    },
    /// 模糊搜索备份中词条
    Search {
        index: usize,
        keyword: String,
    },
}

fn require_dict() -> anyhow::Result<dict::DictFile> {
    let path = config::dict_path();
    if !path.exists() {
        anyhow::bail!("词库文件不存在，请先运行 init");
    }
    dict::DictFile::load(&path)
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Init => {
            config::init_custom_words()?;
            config::init_config()?;
        }
        Command::Add {
            word,
            pinyin,
            weight,
        } => {
            let path = config::dict_path();
            if !path.exists() {
                anyhow::bail!("词库文件不存在，请先运行 init");
            }
            let mut d = dict::DictFile::load(&path)?;
            let entry = dict::WordEntry::new(word, pinyin, weight);
            if d.add(entry) {
                d.save(&path)?;
                println!("已添加词条");
            } else {
                anyhow::bail!("词条已存在");
            }
        }
        Command::Remove { word } => {
            let path = config::dict_path();
            if !path.exists() {
                anyhow::bail!("词库文件不存在，请先运行 init");
            }
            let mut d = dict::DictFile::load(&path)?;
            let removed = d.remove(&word);
            if removed > 0 {
                d.save(&path)?;
                println!("已删除 {} 条词条", removed);
            } else {
                println!("未找到词条 \"{}\"", word);
            }
        }
        Command::List { line_numbers } => {
            let d = require_dict()?;
            if d.list().is_empty() {
                println!("(空)");
            } else {
                for (i, entry) in d.list().iter().enumerate() {
                    if line_numbers {
                        println!(
                            "{:>4}. {}\t{}\t{}",
                            i + 1,
                            entry.word,
                            entry.pinyin,
                            entry.weight
                        );
                    } else {
                        println!("{}\t{}\t{}", entry.word, entry.pinyin, entry.weight);
                    }
                }
            }
        }
        Command::Search { keyword } => {
            let d = require_dict()?;
            let results = d.search(&keyword);
            if results.is_empty() {
                println!("未找到匹配词条");
            } else {
                for entry in &results {
                    println!("{}\t{}\t{}", entry.word, entry.pinyin, entry.weight);
                }
            }
        }
        Command::Cleanup => {
            config::cleanup_config()?;
            if config::confirm("是否重新部署 fcitx5？")? {
                deploy::deploy()?;
            }
        }
        Command::Deploy => deploy::deploy()?,
        Command::Backup(args) => {
            if args.list_backups {
                backup::list_backups()?;
            } else if let Some(idx) = args.reset {
                backup::restore_backup(idx)?;
            } else if let Some(ref val) = args.delete {
                if val == "all" {
                    backup::delete_backup(None)?;
                } else {
                    let idx: usize = val.parse().context("无效的备份序号")?;
                    backup::delete_backup(Some(idx))?;
                }
            } else if let Some(cmd) = args.command {
                match cmd {
                    BackupCommand::List {
                        index,
                        line_numbers,
                    } => backup::show_backup(index, line_numbers)?,
                    BackupCommand::Search { index, keyword } => {
                        backup::search_backup(index, &keyword)?;
                    }
                }
            } else {
                backup::create_backup()?;
            }
        }
    }

    Ok(())
}
