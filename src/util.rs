use std::collections::hash_map::DefaultHasher;
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::io::{self, Read, Write};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

use crate::error::{MaehError, Result};

pub(crate) fn take_arg(args: &mut Vec<String>, name: &str) -> Result<String> {
    if args.is_empty() {
        Err(MaehError::Usage(format!("missing {name}")))
    } else {
        Ok(args.remove(0))
    }
}

pub(crate) fn flag_value(args: &mut Vec<String>, flag: &str, default: &str) -> Result<String> {
    if let Some(index) = args.iter().position(|arg| arg == flag) {
        args.remove(index);
        if index >= args.len() {
            Err(MaehError::Usage(format!("{flag} needs a value")))
        } else {
            Ok(args.remove(index))
        }
    } else {
        Ok(default.to_string())
    }
}

pub(crate) fn flag_present(args: &mut Vec<String>, flag: &str) -> bool {
    if let Some(index) = args.iter().position(|arg| arg == flag) {
        args.remove(index);
        true
    } else {
        false
    }
}

pub(crate) fn join_numbers(values: &[u32]) -> String {
    values
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

pub(crate) fn read_json_stdin() -> Result<Value> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    Ok(serde_json::from_str(input.trim())?)
}

pub(crate) fn write_json(path: &Path, value: &Value) -> Result<()> {
    write_file(path, value.to_string().as_bytes())
}

pub(crate) fn write_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let tmp = path.with_extension("tmp");
    File::create(&tmp)?.write_all(bytes)?;
    fs::rename(tmp, path)?;
    Ok(())
}

pub(crate) fn now() -> String {
    std::env::var("MAEH_NOW").unwrap_or_else(|_| now_epoch().to_string())
}

pub(crate) fn now_epoch() -> u64 {
    if let Ok(value) = std::env::var("MAEH_EPOCH") {
        if let Ok(parsed) = value.parse() {
            return parsed;
        }
    }
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub(crate) fn stable_hash(value: &str) -> String {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub(crate) fn display(path: &Path) -> String {
    path.display().to_string()
}
