use std::fs;
use std::path::{Path, PathBuf};

use maeh::backend::{BackendEnv, BackendKind, BackendSettings};
use serde::{Deserialize, Serialize};

use crate::error::{MaehError, Result};
use crate::util::{display, join_numbers, stable_hash, take_arg, write_file};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct DefaultConfig {
    home: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct Config {
    pub(crate) backend: BackendKind,
    pub(crate) herdr_bin: String,
    pub(crate) tmux_bin: String,
    pub(crate) tmux_session: String,
    pub(crate) include_editor: bool,
    pub(crate) focus: bool,
    pub(crate) primary_agent_cmd: String,
    pub(crate) critic_agent_cmd: String,
    pub(crate) editor_cmd: String,
    pub(crate) context_switch_cap: u64,
    pub(crate) review_cap: u64,
    pub(crate) board_ttl_intake_secs: u64,
    pub(crate) board_ttl_revamp_secs: u64,
    pub(crate) task_capsule_max_chars: usize,
    pub(crate) work_start_hour: u32,
    pub(crate) work_end_hour: u32,
    pub(crate) workdays: Vec<u32>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            backend: BackendKind::Auto,
            herdr_bin: "herdr".to_string(),
            tmux_bin: "tmux".to_string(),
            tmux_session: "maeh".to_string(),
            include_editor: true,
            focus: false,
            primary_agent_cmd: "codex".to_string(),
            critic_agent_cmd: "codex".to_string(),
            editor_cmd: "vi".to_string(),
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

pub(crate) fn resolve_home() -> Result<PathBuf> {
    if let Some(home) = std::env::var_os("MAEH_HOME") {
        return Ok(PathBuf::from(home));
    }
    if let Some(home) = default_home()? {
        return Ok(home);
    }
    if let Some(home) = std::env::var_os("HOME") {
        return Ok(PathBuf::from(home).join(".maeh"));
    }
    Ok(PathBuf::from(".maeh"))
}

pub(crate) fn default_config_path() -> PathBuf {
    if let Some(path) = std::env::var_os("MAEH_CONFIG") {
        return PathBuf::from(path);
    }
    if let Some(home) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(home).join("maeh").join("config.toml");
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home)
            .join(".config")
            .join("maeh")
            .join("config.toml");
    }
    PathBuf::from(".config/maeh/config.toml")
}

fn default_home() -> Result<Option<PathBuf>> {
    let path = default_config_path();
    if !path.exists() {
        return Ok(None);
    }
    let config: DefaultConfig = toml::from_str(&fs::read_to_string(&path)?)?;
    Ok(config.home.map(|home| configured_path(&path, &home)))
}

fn configured_path(config: &Path, value: &str) -> PathBuf {
    let path = expand_home(value);
    if path.is_absolute() {
        path
    } else {
        config.parent().unwrap_or(Path::new(".")).join(path)
    }
}

fn expand_home(value: &str) -> PathBuf {
    if value == "~" {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home);
        }
    }
    if let Some(rest) = value.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(value)
}

pub(crate) fn absolute_path(path: &Path) -> Result<PathBuf> {
    let path = expand_home(&display(path));
    if path.is_absolute() {
        Ok(path)
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

fn write_default_home(home: &Path) -> Result<()> {
    let config = DefaultConfig {
        home: Some(display(&absolute_path(home)?)),
    };
    write_file(
        &default_config_path(),
        toml::to_string_pretty(&config)?.as_bytes(),
    )
}

pub(crate) fn config_path(home: &Path) -> PathBuf {
    home.join("config.toml")
}

pub(crate) fn ledger_dir(home: &Path) -> PathBuf {
    home.join("ledger")
}

pub(crate) fn state_path(home: &Path) -> PathBuf {
    home.join("state.json")
}

pub(crate) fn board_cache_path(home: &Path, key: &str) -> PathBuf {
    home.join("board-cache").join(format!("{key}.json"))
}

pub(crate) fn capsule_path(home: &Path, url: &str) -> PathBuf {
    home.join("task-capsules")
        .join(format!("{}.json", stable_hash(url)))
}

pub(crate) fn config_command(home: &Path, args: &mut Vec<String>) -> Result<()> {
    match take_arg(args, "config command")?.as_str() {
        "emit" => emit_config(home),
        "path" => {
            println!("{}", display(&config_path(home)));
            Ok(())
        }
        "set-home" => set_default_home(home, args),
        "show" => show_config(home),
        other => Err(MaehError::Usage(format!("unknown config command {other}"))),
    }
}

fn set_default_home(home: &Path, args: &mut Vec<String>) -> Result<()> {
    let selected = if args.is_empty() {
        home.to_path_buf()
    } else {
        PathBuf::from(take_arg(args, "home path")?)
    };
    let selected = absolute_path(&selected)?;
    write_default_home(&selected)?;
    println!("default home: {}", display(&selected));
    println!("default config: {}", display(&default_config_path()));
    Ok(())
}

pub(crate) fn read_config(home: &Path) -> Result<Config> {
    let path = config_path(home);
    let mut config = if path.exists() {
        toml::from_str(&fs::read_to_string(path)?)?
    } else {
        Config::default()
    };
    apply_config_env(&mut config);
    Ok(config)
}

fn apply_config_env(config: &mut Config) {
    if let Some(value) = non_empty_env("MAEH_INCLUDE_EDITOR") {
        config.include_editor = parse_bool(&value, config.include_editor);
    }
    if let Some(value) = non_empty_env("MAEH_FOCUS") {
        config.focus = parse_bool(&value, config.focus);
    }
    if let Some(value) = non_empty_env("MAEH_PRIMARY_AGENT_CMD") {
        config.primary_agent_cmd = value;
    }
    if let Some(value) = non_empty_env("MAEH_CRITIC_AGENT_CMD") {
        config.critic_agent_cmd = value;
    }
    if let Some(value) = non_empty_env("MAEH_EDITOR_CMD") {
        config.editor_cmd = value;
    }
}

pub(crate) fn backend_settings_for_config(config: &Config) -> Result<BackendSettings> {
    backend_settings_for_config_env(config, &BackendEnv::from_env()?)
}

pub(crate) fn backend_settings_for_config_env(
    config: &Config,
    env: &BackendEnv,
) -> Result<BackendSettings> {
    Ok(BackendSettings::resolve(
        config.backend,
        &config.herdr_bin,
        &config.tmux_bin,
        &config.tmux_session,
        env,
    ))
}

fn non_empty_env(name: &str) -> Option<String> {
    match std::env::var(name) {
        Ok(value) if !value.is_empty() => Some(value),
        _ => None,
    }
}

pub(crate) fn parse_bool(value: &str, fallback: bool) -> bool {
    match value {
        "1" | "true" | "yes" | "on" => true,
        "0" | "false" | "no" | "off" => false,
        _ => fallback,
    }
}

pub(crate) fn backend_settings(home: &Path) -> Result<BackendSettings> {
    backend_settings_for_config(&read_config(home)?)
}

pub(crate) fn print_backend_resolution(settings: &BackendSettings) {
    println!("  requested backend: {}", settings.requested);
    println!("  selected backend: {}", settings.selected);
    println!("  herdr bin: {}", settings.herdr_bin);
    println!("  tmux bin: {}", settings.tmux_bin);
    println!("  tmux session: {}", settings.tmux_session);
}

fn show_config(home: &Path) -> Result<()> {
    let config = read_config(home)?;
    let settings = backend_settings_for_config(&config)?;
    println!("maeh config");
    println!("  home: {}", display(home));
    println!("  backend: {}", config.backend);
    print_backend_resolution(&settings);
    println!("  include editor: {}", config.include_editor);
    println!("  focus: {}", config.focus);
    println!("  primary agent cmd: {}", config.primary_agent_cmd);
    println!("  critic agent cmd: {}", config.critic_agent_cmd);
    println!("  editor cmd: {}", config.editor_cmd);
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
    println!("MAEH_HERDR_BIN={}", config.herdr_bin);
    println!("MAEH_TMUX_BIN={}", config.tmux_bin);
    println!("MAEH_TMUX_SESSION={}", config.tmux_session);
    println!("MAEH_INCLUDE_EDITOR={}", config.include_editor);
    println!("MAEH_FOCUS={}", config.focus);
    println!("MAEH_PRIMARY_AGENT_CMD={}", config.primary_agent_cmd);
    println!("MAEH_CRITIC_AGENT_CMD={}", config.critic_agent_cmd);
    println!("MAEH_EDITOR_CMD={}", config.editor_cmd);
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
