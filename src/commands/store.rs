use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::config::{board_cache_path, capsule_path, read_config, Config};
use crate::error::{MaehError, Result};
use crate::util::{
    display, flag_present, flag_value, now, now_epoch, read_json_stdin, take_arg, write_json,
};

pub(crate) fn board_cache_command(home: &Path, args: &mut Vec<String>) -> Result<()> {
    match take_arg(args, "board-cache command")?.as_str() {
        "put" => put_board_cache(home, &flag_value(args, "--key", "intake")?),
        "get" => {
            let key = flag_value(args, "--key", "intake")?;
            let stale = flag_present(args, "--stale");
            get_board_cache(home, &key, stale)
        }
        other => Err(MaehError::Usage(format!(
            "unknown board-cache command {other}"
        ))),
    }
}

fn put_board_cache(home: &Path, key: &str) -> Result<()> {
    let board = read_json_stdin()?;
    let path = board_cache_path(home, key);
    let payload = json!({"cached_at": now(), "epoch": now_epoch(), "board": board});
    write_json(&path, &payload)?;
    println!("board cache stored");
    println!("  key: {key}");
    println!("  file: {}", display(&path));
    Ok(())
}

fn get_board_cache(home: &Path, key: &str, stale: bool) -> Result<()> {
    let config = read_config(home)?;
    let path = board_cache_path(home, key);
    let raw = fs::read_to_string(&path).map_err(|_| MaehError::CacheMiss(key.to_string()))?;
    let cache: Value = serde_json::from_str(&raw)?;
    let age = now_epoch().saturating_sub(cache["epoch"].as_u64().unwrap_or(0));
    if !stale && age > board_ttl(&config, key) {
        return Err(MaehError::CacheMiss(key.to_string()));
    }
    println!("{}", cache["board"]);
    Ok(())
}

fn board_ttl(config: &Config, key: &str) -> u64 {
    if key == "revamp" {
        config.board_ttl_revamp_secs
    } else {
        config.board_ttl_intake_secs
    }
}

pub(crate) fn capsule_command(home: &Path, args: &mut Vec<String>) -> Result<()> {
    match take_arg(args, "capsule command")?.as_str() {
        "put" => {
            let url = take_arg(args, "url")?;
            let edited = flag_value(args, "--edited", "")?;
            put_capsule(home, &url, &edited)
        }
        "get" => {
            let url = take_arg(args, "url")?;
            let edited = flag_value(args, "--edited", "")?;
            get_capsule(home, &url, &edited, false)
        }
        "prompt" => {
            let url = take_arg(args, "url")?;
            let edited = flag_value(args, "--edited", "")?;
            get_capsule(home, &url, &edited, true)
        }
        other => Err(MaehError::Usage(format!("unknown capsule command {other}"))),
    }
}

fn put_capsule(home: &Path, url: &str, edited: &str) -> Result<()> {
    let capsule = read_json_stdin()?;
    let raw = capsule.to_string();
    let max = read_config(home)?.task_capsule_max_chars;
    if raw.chars().count() > max {
        return Err(MaehError::CapsuleTooLarge {
            actual: raw.chars().count(),
            max,
        });
    }
    let path = capsule_path(home, url);
    let payload = json!({"cached_at": now(), "epoch": now_epoch(), "url": url, "source_last_edited": edited, "capsule": capsule});
    write_json(&path, &payload)?;
    println!("capsule stored");
    println!("  url: {url}");
    println!("  file: {}", display(&path));
    Ok(())
}

fn get_capsule(home: &Path, url: &str, edited: &str, prompt: bool) -> Result<()> {
    let path = capsule_path(home, url);
    let raw = fs::read_to_string(&path).map_err(|_| MaehError::CacheMiss(url.to_string()))?;
    let payload: Value = serde_json::from_str(&raw)?;
    if !edited.is_empty() && payload["source_last_edited"].as_str().unwrap_or("") != edited {
        return Err(MaehError::CacheMiss(url.to_string()));
    }
    let capsule = payload["capsule"].to_string();
    if prompt {
        println!("Task capsule");
        println!("```json");
        println!("{capsule}");
        println!("```");
    } else {
        println!("{capsule}");
    }
    Ok(())
}

pub(crate) fn prompt_command(args: &mut Vec<String>) -> Result<()> {
    match take_arg(args, "prompt command")?.as_str() {
        "kickoff" => {
            let url = flag_value(args, "--url", "")?;
            let capsule_file = if args.iter().any(|arg| arg == "--capsule-file") {
                Some(PathBuf::from(flag_value(args, "--capsule-file", "")?))
            } else {
                None
            };
            kickoff_prompt(&url, capsule_file.as_deref())
        }
        other => Err(MaehError::Usage(format!("unknown prompt command {other}"))),
    }
}

fn kickoff_prompt(url: &str, capsule_file: Option<&Path>) -> Result<()> {
    let capsule = match capsule_file {
        Some(path) => fs::read_to_string(path)?,
        None => "{}".to_string(),
    };
    println!("Maeh kickoff");
    println!("  task: {url}");
    println!(
        "  instruction: use the capsule first; fetch tracker context only if stale or insufficient"
    );
    println!("  guardrail: plan with the critic before writing code");
    println!("Task capsule");
    println!("```json");
    println!("{}", capsule.trim());
    println!("```");
    Ok(())
}
