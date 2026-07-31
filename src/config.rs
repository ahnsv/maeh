use std::fs;
use std::path::{Path, PathBuf};

use maeh::backend::{BackendEnv, BackendKind, BackendSettings};
use serde::{Deserialize, Serialize};

use crate::error::{MaehError, Result};
use crate::util::{display, join_numbers, stable_hash, take_arg, write_file};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct DefaultConfig {
    paths: DefaultPathsConfig,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
struct DefaultPathsConfig {
    home: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct Config {
    pub(crate) backend: BackendConfig,
    pub(crate) layout: LayoutConfig,
    pub(crate) agents: AgentConfig,
    pub(crate) limits: LimitsConfig,
    pub(crate) board_cache: BoardCacheConfig,
    pub(crate) task_capsules: TaskCapsuleConfig,
    pub(crate) work_hours: WorkHoursConfig,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct BackendConfig {
    pub(crate) kind: BackendKind,
    pub(crate) herdr_bin: String,
    pub(crate) tmux_bin: String,
    pub(crate) tmux_session: String,
}

impl Default for BackendConfig {
    fn default() -> Self {
        Self {
            kind: BackendKind::Auto,
            herdr_bin: "herdr".to_string(),
            tmux_bin: "tmux".to_string(),
            tmux_session: "maeh".to_string(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct LayoutConfig {
    pub(crate) include_editor: bool,
    pub(crate) focus: bool,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            include_editor: true,
            focus: false,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct AgentConfig {
    pub(crate) primary_cmd: String,
    pub(crate) critic_cmd: String,
    pub(crate) editor_cmd: String,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            primary_cmd: "codex".to_string(),
            critic_cmd: "codex".to_string(),
            editor_cmd: "vi".to_string(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct LimitsConfig {
    pub(crate) context_switch_cap: u64,
    pub(crate) review_cap: u64,
}

impl Default for LimitsConfig {
    fn default() -> Self {
        Self {
            context_switch_cap: 3,
            review_cap: 5,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct BoardCacheConfig {
    pub(crate) intake_ttl_secs: u64,
    pub(crate) revamp_ttl_secs: u64,
}

impl Default for BoardCacheConfig {
    fn default() -> Self {
        Self {
            intake_ttl_secs: 3_600,
            revamp_ttl_secs: 10_800,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct TaskCapsuleConfig {
    pub(crate) max_chars: usize,
}

impl Default for TaskCapsuleConfig {
    fn default() -> Self {
        Self { max_chars: 1_800 }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct WorkHoursConfig {
    pub(crate) start_hour: u32,
    pub(crate) end_hour: u32,
    pub(crate) workdays: Vec<u32>,
}

impl Default for WorkHoursConfig {
    fn default() -> Self {
        Self {
            start_hour: 9,
            end_hour: 17,
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
    let mut value: toml::Value = toml::from_str(&fs::read_to_string(&path)?)?;
    normalize_default_config(&mut value);
    let config: DefaultConfig = value.try_into()?;
    Ok(config.paths.home.map(|home| configured_path(&path, &home)))
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
        paths: DefaultPathsConfig {
            home: Some(display(&absolute_path(home)?)),
        },
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
        let mut value: toml::Value = toml::from_str(&fs::read_to_string(path)?)?;
        normalize_config(&mut value);
        value.try_into()?
    } else {
        Config::default()
    };
    apply_config_env(&mut config);
    Ok(config)
}

fn normalize_default_config(value: &mut toml::Value) {
    let Some(table) = value.as_table_mut() else {
        return;
    };
    move_legacy_key(table, "home", "paths", "home");
}

fn normalize_config(value: &mut toml::Value) {
    let Some(table) = value.as_table_mut() else {
        return;
    };
    move_legacy_key(table, "backend", "backend", "kind");
    move_legacy_key(table, "herdr_bin", "backend", "herdr_bin");
    move_legacy_key(table, "tmux_bin", "backend", "tmux_bin");
    move_legacy_key(table, "tmux_session", "backend", "tmux_session");
    move_legacy_key(table, "include_editor", "layout", "include_editor");
    move_legacy_key(table, "focus", "layout", "focus");
    move_legacy_key(table, "primary_agent_cmd", "agents", "primary_cmd");
    move_legacy_key(table, "critic_agent_cmd", "agents", "critic_cmd");
    move_legacy_key(table, "editor_cmd", "agents", "editor_cmd");
    move_legacy_key(table, "context_switch_cap", "limits", "context_switch_cap");
    move_legacy_key(table, "review_cap", "limits", "review_cap");
    move_legacy_key(
        table,
        "board_ttl_intake_secs",
        "board_cache",
        "intake_ttl_secs",
    );
    move_legacy_key(
        table,
        "board_ttl_revamp_secs",
        "board_cache",
        "revamp_ttl_secs",
    );
    move_legacy_key(
        table,
        "task_capsule_max_chars",
        "task_capsules",
        "max_chars",
    );
    move_legacy_key(table, "work_start_hour", "work_hours", "start_hour");
    move_legacy_key(table, "work_end_hour", "work_hours", "end_hour");
    move_legacy_key(table, "workdays", "work_hours", "workdays");
}

fn move_legacy_key(table: &mut toml::Table, key: &str, section: &str, field: &str) {
    let Some(value) = table.remove(key) else {
        return;
    };
    if key == section && value.is_table() {
        table.insert(key.to_string(), value);
        return;
    }
    let section = table
        .entry(section.to_string())
        .or_insert_with(|| toml::Value::Table(toml::Table::new()));
    if let toml::Value::Table(section) = section {
        section.entry(field.to_string()).or_insert(value);
    }
}

fn apply_config_env(config: &mut Config) {
    if let Some(value) = non_empty_env("MAEH_INCLUDE_EDITOR") {
        config.layout.include_editor = parse_bool(&value, config.layout.include_editor);
    }
    if let Some(value) = non_empty_env("MAEH_FOCUS") {
        config.layout.focus = parse_bool(&value, config.layout.focus);
    }
    if let Some(value) = non_empty_env("MAEH_PRIMARY_AGENT_CMD") {
        config.agents.primary_cmd = value;
    }
    if let Some(value) = non_empty_env("MAEH_CRITIC_AGENT_CMD") {
        config.agents.critic_cmd = value;
    }
    if let Some(value) = non_empty_env("MAEH_EDITOR_CMD") {
        config.agents.editor_cmd = value;
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
        config.backend.kind,
        &config.backend.herdr_bin,
        &config.backend.tmux_bin,
        &config.backend.tmux_session,
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
    println!("  backend: {}", config.backend.kind);
    print_backend_resolution(&settings);
    println!("  include editor: {}", config.layout.include_editor);
    println!("  focus: {}", config.layout.focus);
    println!("  primary agent cmd: {}", config.agents.primary_cmd);
    println!("  critic agent cmd: {}", config.agents.critic_cmd);
    println!("  editor cmd: {}", config.agents.editor_cmd);
    println!("  context switch cap: {}", config.limits.context_switch_cap);
    println!("  review cap: {}", config.limits.review_cap);
    println!(
        "  board ttl intake: {}s",
        config.board_cache.intake_ttl_secs
    );
    println!(
        "  board ttl revamp: {}s",
        config.board_cache.revamp_ttl_secs
    );
    println!("  capsule max chars: {}", config.task_capsules.max_chars);
    println!(
        "  work hours: {}-{}",
        config.work_hours.start_hour, config.work_hours.end_hour
    );
    println!("  workdays: {}", join_numbers(&config.work_hours.workdays));
    Ok(())
}

fn emit_config(home: &Path) -> Result<()> {
    let config = read_config(home)?;
    println!("MAEH_BACKEND={}", config.backend.kind);
    println!("MAEH_HERDR_BIN={}", config.backend.herdr_bin);
    println!("MAEH_TMUX_BIN={}", config.backend.tmux_bin);
    println!("MAEH_TMUX_SESSION={}", config.backend.tmux_session);
    println!("MAEH_INCLUDE_EDITOR={}", config.layout.include_editor);
    println!("MAEH_FOCUS={}", config.layout.focus);
    println!("MAEH_PRIMARY_AGENT_CMD={}", config.agents.primary_cmd);
    println!("MAEH_CRITIC_AGENT_CMD={}", config.agents.critic_cmd);
    println!("MAEH_EDITOR_CMD={}", config.agents.editor_cmd);
    println!(
        "MAEH_CONTEXT_SWITCH_CAP={}",
        config.limits.context_switch_cap
    );
    println!("MAEH_REVIEW_CAP={}", config.limits.review_cap);
    println!(
        "MAEH_BOARD_TTL_INTAKE={}",
        config.board_cache.intake_ttl_secs
    );
    println!(
        "MAEH_BOARD_TTL_REVAMP={}",
        config.board_cache.revamp_ttl_secs
    );
    println!(
        "MAEH_TASK_CAPSULE_MAX_CHARS={}",
        config.task_capsules.max_chars
    );
    Ok(())
}
