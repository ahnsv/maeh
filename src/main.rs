use std::collections::{hash_map::DefaultHasher, BTreeMap};
use std::ffi::OsString;
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{Datelike, Timelike};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Debug, Error)]
enum MaehError {
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("toml decode: {0}")]
    TomlDecode(#[from] toml::de::Error),
    #[error("toml encode: {0}")]
    TomlEncode(#[from] toml::ser::Error),
    #[error("cache miss: {0}")]
    CacheMiss(String),
    #[error("capsule too large: {actual} chars > {max} chars")]
    CapsuleTooLarge { actual: usize, max: usize },
    #[error("usage: {0}")]
    Usage(String),
}

type Result<T> = std::result::Result<T, MaehError>;
type State = BTreeMap<String, BTreeMap<String, String>>;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Config {
    backend: String,
    context_switch_cap: u64,
    review_cap: u64,
    board_ttl_intake_secs: u64,
    board_ttl_revamp_secs: u64,
    task_capsule_max_chars: usize,
    work_start_hour: u32,
    work_end_hour: u32,
    workdays: Vec<u32>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            backend: "auto".to_string(),
            context_switch_cap: 3,
            review_cap: 5,
            board_ttl_intake_secs: 3_600,
            board_ttl_revamp_secs: 10_800,
            task_capsule_max_chars: 1_800,
            work_start_hour: 9,
            work_end_hour: 17,
            workdays: vec![1, 2, 3, 4, 5],
        }
    }
}

fn main() {
    if let Err(err) = run(std::env::args_os().skip(1).collect()) {
        eprintln!("maeh error: {err}");
        std::process::exit(1);
    }
}

fn run(args: Vec<OsString>) -> Result<()> {
    let mut args = args
        .into_iter()
        .map(|arg| arg.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    if args
        .first()
        .is_some_and(|arg| arg == "--help" || arg == "-h")
    {
        print_help();
        return Ok(());
    }
    let home = if args.first().is_some_and(|arg| arg == "--home") {
        args.remove(0);
        PathBuf::from(take_arg(&mut args, "home path")?)
    } else {
        resolve_home()
    };
    dispatch(&home, &mut args)
}

fn dispatch(home: &Path, args: &mut Vec<String>) -> Result<()> {
    let command = take_arg(args, "command")?;
    match command.as_str() {
        "init" => init(home),
        "config" => config_command(home, args),
        "ledger" => ledger_command(home, args),
        "state" => state_command(home, args),
        "board-cache" => board_cache_command(home, args),
        "capsule" => capsule_command(home, args),
        "prompt" => prompt_command(args),
        "statusline" => statusline(home),
        "work-hours" => work_hours(home),
        "doctor" => doctor(home),
        "selftest" => selftest(home),
        other => Err(MaehError::Usage(format!("unknown command {other}"))),
    }
}

fn print_help() {
    println!("Typed orchestration CLI for hmph and Herdr agents");
    println!();
    println!("Usage: maeh [--home PATH] <command>");
    println!();
    println!("Commands:");
    println!("  init          create local state directories and config");
    println!("  config        path, show, or emit config");
    println!("  ledger        append or list JSONL spans");
    println!("  state         tag, untag, get, list, worktree, delete-slot");
    println!("  board-cache   put or get tracker board snapshots");
    println!("  capsule       put, get, or prompt compact task context");
    println!("  prompt        render kickoff prompts");
    println!("  statusline    print compact pool status");
    println!("  work-hours    evaluate configured work-hour guard");
    println!("  doctor        debug paths, config, backend, and env");
    println!("  selftest      validate local config/state readability");
}

fn print_state_help() {
    println!("Manage local slot state");
    println!("Usage: maeh state <tag|untag|get|list|worktree|delete-slot>");
}

fn take_arg(args: &mut Vec<String>, name: &str) -> Result<String> {
    if args.is_empty() {
        Err(MaehError::Usage(format!("missing {name}")))
    } else {
        Ok(args.remove(0))
    }
}

fn flag_value(args: &mut Vec<String>, flag: &str, default: &str) -> Result<String> {
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

fn flag_present(args: &mut Vec<String>, flag: &str) -> bool {
    if let Some(index) = args.iter().position(|arg| arg == flag) {
        args.remove(index);
        true
    } else {
        false
    }
}

fn resolve_home() -> PathBuf {
    if let Some(home) = std::env::var_os("MAEH_HOME") {
        return PathBuf::from(home);
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".maeh");
    }
    PathBuf::from(".maeh")
}

fn config_path(home: &Path) -> PathBuf {
    home.join("config.toml")
}

fn ledger_dir(home: &Path) -> PathBuf {
    home.join("ledger")
}

fn state_path(home: &Path) -> PathBuf {
    home.join("state.json")
}

fn board_cache_path(home: &Path, key: &str) -> PathBuf {
    home.join("board-cache").join(format!("{key}.json"))
}

fn capsule_path(home: &Path, url: &str) -> PathBuf {
    home.join("task-capsules")
        .join(format!("{}.json", stable_hash(url)))
}

fn init(home: &Path) -> Result<()> {
    fs::create_dir_all(ledger_dir(home))?;
    fs::create_dir_all(home.join("board-cache"))?;
    fs::create_dir_all(home.join("task-capsules"))?;
    let path = config_path(home);
    if !path.exists() {
        let config_text = toml::to_string_pretty(&Config::default())?;
        write_file(&path, config_text.as_bytes())?;
    }
    println!("maeh");
    println!("  created: {}", display(home));
    println!("  config: {}", display(&path));
    println!("  ledger: {}", display(&ledger_dir(home)));
    Ok(())
}

fn config_command(home: &Path, args: &mut Vec<String>) -> Result<()> {
    match take_arg(args, "config command")?.as_str() {
        "emit" => emit_config(home),
        "path" => {
            println!("{}", display(&config_path(home)));
            Ok(())
        }
        "show" => show_config(home),
        other => Err(MaehError::Usage(format!("unknown config command {other}"))),
    }
}

fn read_config(home: &Path) -> Result<Config> {
    let path = config_path(home);
    if path.exists() {
        Ok(toml::from_str(&fs::read_to_string(path)?)?)
    } else {
        Ok(Config::default())
    }
}

fn show_config(home: &Path) -> Result<()> {
    let config = read_config(home)?;
    println!("maeh config");
    println!("  home: {}", display(home));
    println!("  backend: {}", config.backend);
    println!("  context switch cap: {}", config.context_switch_cap);
    println!("  review cap: {}", config.review_cap);
    println!("  board ttl intake: {}s", config.board_ttl_intake_secs);
    println!("  board ttl revamp: {}s", config.board_ttl_revamp_secs);
    println!("  capsule max chars: {}", config.task_capsule_max_chars);
    println!(
        "  work hours: {}-{}",
        config.work_start_hour, config.work_end_hour
    );
    println!("  workdays: {}", join_numbers(&config.workdays));
    Ok(())
}

fn emit_config(home: &Path) -> Result<()> {
    let config = read_config(home)?;
    println!("MAEH_BACKEND={}", config.backend);
    println!("MAEH_CONTEXT_SWITCH_CAP={}", config.context_switch_cap);
    println!("MAEH_REVIEW_CAP={}", config.review_cap);
    println!("MAEH_BOARD_TTL_INTAKE={}", config.board_ttl_intake_secs);
    println!("MAEH_BOARD_TTL_REVAMP={}", config.board_ttl_revamp_secs);
    println!(
        "MAEH_TASK_CAPSULE_MAX_CHARS={}",
        config.task_capsule_max_chars
    );
    Ok(())
}

fn ledger_command(home: &Path, args: &mut Vec<String>) -> Result<()> {
    match take_arg(args, "ledger command")?.as_str() {
        "append" => {
            let loop_name = flag_value(args, "--loop", "")?;
            let event = flag_value(args, "--event", "")?;
            let target = flag_value(args, "--target", "")?;
            let data = flag_value(args, "--data", "{}")?;
            append_ledger(home, &loop_name, &event, &target, &data)
        }
        "list" => {
            let loop_name = flag_value(args, "--loop", "")?;
            list_ledger(home, &loop_name)
        }
        other => Err(MaehError::Usage(format!("unknown ledger command {other}"))),
    }
}

fn append_ledger(
    home: &Path,
    loop_name: &str,
    event: &str,
    target: &str,
    data: &str,
) -> Result<()> {
    let data: Value = serde_json::from_str(data)?;
    let path = ledger_dir(home).join(format!("{loop_name}.jsonl"));
    fs::create_dir_all(ledger_dir(home))?;
    let row =
        json!({"ts": now(), "loop": loop_name, "event": event, "target": target, "data": data});
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)?;
    writeln!(file, "{row}")?;
    println!("ledger appended");
    println!("  file: {}", display(&path));
    println!("  event: {event}");
    println!("  target: {target}");
    Ok(())
}

fn list_ledger(home: &Path, loop_name: &str) -> Result<()> {
    let path = ledger_dir(home).join(format!("{loop_name}.jsonl"));
    let content = fs::read_to_string(&path).unwrap_or_default();
    for line in content.lines() {
        let row: Value = serde_json::from_str(line)?;
        println!(
            "{} {} {} {}",
            row["ts"].as_str().unwrap_or(""),
            row["event"].as_str().unwrap_or(""),
            row["target"].as_str().unwrap_or(""),
            row["data"]
        );
    }
    Ok(())
}

fn state_command(home: &Path, args: &mut Vec<String>) -> Result<()> {
    if args
        .first()
        .is_some_and(|arg| arg == "--help" || arg == "-h")
    {
        print_state_help();
        return Ok(());
    }
    match take_arg(args, "state command")?.as_str() {
        "tag" => state_tag(
            home,
            &take_arg(args, "slot")?,
            &take_arg(args, "key")?,
            &take_arg(args, "value")?,
        ),
        "untag" => state_untag(home, &take_arg(args, "slot")?, &take_arg(args, "key")?),
        "get" => state_get(home, &take_arg(args, "slot")?, &take_arg(args, "key")?),
        "list" => state_list(home),
        "worktree" => state_get(home, &take_arg(args, "slot")?, "worktree"),
        "delete-slot" => state_delete_slot(home, &take_arg(args, "slot")?),
        other => Err(MaehError::Usage(format!("unknown state command {other}"))),
    }
}

fn read_state(home: &Path) -> Result<State> {
    let path = state_path(home);
    if path.exists() {
        Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
    } else {
        Ok(State::new())
    }
}

fn write_state(home: &Path, state: &State) -> Result<()> {
    write_json(&state_path(home), &serde_json::to_value(state)?)
}

fn state_tag(home: &Path, slot: &str, key: &str, value: &str) -> Result<()> {
    let mut state = read_state(home)?;
    state
        .entry(slot.to_string())
        .or_default()
        .insert(key.to_string(), value.to_string());
    write_state(home, &state)?;
    println!("state tagged");
    println!("  slot: {slot}");
    println!("  {key}: {value}");
    Ok(())
}

fn state_untag(home: &Path, slot: &str, key: &str) -> Result<()> {
    let mut state = read_state(home)?;
    if let Some(entry) = state.get_mut(slot) {
        entry.remove(key);
    }
    write_state(home, &state)?;
    println!("state untagged");
    println!("  slot: {slot}");
    println!("  key: {key}");
    Ok(())
}

fn state_get(home: &Path, slot: &str, key: &str) -> Result<()> {
    let state = read_state(home)?;
    let value = state
        .get(slot)
        .and_then(|entry| entry.get(key))
        .ok_or_else(|| MaehError::CacheMiss(format!("{slot}:{key}")))?;
    println!("{value}");
    Ok(())
}

fn state_list(home: &Path) -> Result<()> {
    for (slot, entry) in read_state(home)? {
        println!(
            "{}\t{}\t{}\t{}\t{}",
            slot,
            entry.get("task_url").map_or("", String::as_str),
            entry.get("status").map_or("none", String::as_str),
            entry.get("snooze_until").map_or("0", String::as_str),
            entry.get("worktree").map_or("", String::as_str)
        );
    }
    Ok(())
}

fn state_delete_slot(home: &Path, slot: &str) -> Result<()> {
    let mut state = read_state(home)?;
    state.remove(slot);
    write_state(home, &state)?;
    println!("state slot deleted");
    println!("  slot: {slot}");
    Ok(())
}

fn board_cache_command(home: &Path, args: &mut Vec<String>) -> Result<()> {
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

fn capsule_command(home: &Path, args: &mut Vec<String>) -> Result<()> {
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

fn prompt_command(args: &mut Vec<String>) -> Result<()> {
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

fn doctor(home: &Path) -> Result<()> {
    let config = read_config(home)?;
    let config_state = if config_path(home).exists() {
        "ok"
    } else {
        "missing"
    };
    let herdr_state = if std::env::var_os("HERDR_ENV").is_some() {
        "detected"
    } else {
        "not-detected"
    };
    let debug_state = if std::env::var_os("MAEH_DEBUG").is_some() {
        "on"
    } else {
        "off"
    };
    println!("maeh doctor");
    println!("  home: {}", display(home));
    println!("  config: {config_state}");
    println!("  ledger: {}", display(&ledger_dir(home)));
    println!("  backend: {}", config.backend);
    println!("  herdr: {herdr_state}");
    println!("  maeh debug: {debug_state}");
    Ok(())
}

fn statusline(home: &Path) -> Result<()> {
    let config = read_config(home)?;
    let mut work = 0;
    let mut review = 0;
    for entry in read_state(home)?.into_values() {
        match entry.get("status").map(String::as_str) {
            Some("active" | "blocked") => work += 1,
            Some("review") => review += 1,
            _ => {}
        }
    }
    println!(
        "maeh W:{}/{} R:{}/{}",
        work, config.context_switch_cap, review, config.review_cap
    );
    Ok(())
}

fn work_hours(home: &Path) -> Result<()> {
    let config = read_config(home)?;
    let (dow, hour) = current_dow_hour();
    let active = config.workdays.contains(&dow)
        && hour >= config.work_start_hour
        && hour < config.work_end_hour;
    println!("work-hours");
    println!("  day: {dow}");
    println!("  hour: {hour}");
    println!("  active: {active}");
    Ok(())
}

fn current_dow_hour() -> (u32, u32) {
    let now = chrono::Local::now();
    let dow = match std::env::var("MAEH_DOW") {
        Ok(value) => value
            .parse()
            .unwrap_or_else(|_| now.weekday().number_from_monday()),
        Err(_) => now.weekday().number_from_monday(),
    };
    let hour = match std::env::var("MAEH_HOUR") {
        Ok(value) => value.parse().unwrap_or_else(|_| now.hour()),
        Err(_) => now.hour(),
    };
    (dow, hour)
}

fn selftest(home: &Path) -> Result<()> {
    let _ = read_config(home)?;
    let _ = read_state(home)?;
    println!("maeh selftest");
    println!("  config: ok");
    println!("  state: ok");
    Ok(())
}

fn join_numbers(values: &[u32]) -> String {
    values
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn read_json_stdin() -> Result<Value> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    Ok(serde_json::from_str(input.trim())?)
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    write_file(path, value.to_string().as_bytes())
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(parent)?;
    let tmp = path.with_extension("tmp");
    File::create(&tmp)?.write_all(bytes)?;
    fs::rename(tmp, path)?;
    Ok(())
}

fn now() -> String {
    std::env::var("MAEH_NOW").unwrap_or_else(|_| now_epoch().to_string())
}

fn now_epoch() -> u64 {
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

fn stable_hash(value: &str) -> String {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

fn display(path: &Path) -> String {
    path.display().to_string()
}
