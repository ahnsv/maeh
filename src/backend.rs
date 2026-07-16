use std::collections::BTreeMap;
use std::fmt;
use std::io;
use std::path::PathBuf;
use std::process::Command;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BackendKind {
    #[default]
    Auto,
    Herdr,
    Tmux,
}

impl BackendKind {
    pub fn resolve(self, env: &BackendEnv) -> Self {
        match self {
            Self::Auto if env.herdr_session => Self::Herdr,
            Self::Auto => Self::Tmux,
            kind => kind,
        }
    }
}

impl fmt::Display for BackendKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Auto => "auto",
            Self::Herdr => "herdr",
            Self::Tmux => "tmux",
        })
    }
}

impl std::str::FromStr for BackendKind {
    type Err = BackendError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "auto" => Ok(Self::Auto),
            "herdr" => Ok(Self::Herdr),
            "tmux" => Ok(Self::Tmux),
            other => Err(BackendError::InvalidBackend(other.to_string())),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackendEnv {
    pub backend_override: Option<BackendKind>,
    pub herdr_session: bool,
    pub herdr_bin: Option<String>,
    pub tmux_bin: Option<String>,
}

impl BackendEnv {
    pub fn from_env() -> Result<Self, BackendError> {
        let backend_override = match std::env::var("MAEH_BACKEND") {
            Ok(value) if !value.is_empty() => Some(value.parse()?),
            _ => None,
        };
        Ok(Self {
            backend_override,
            herdr_session: std::env::var_os("HERDR_ENV").is_some()
                || std::env::var_os("HERDR_SOCKET_PATH").is_some(),
            herdr_bin: non_empty_env("MAEH_HERDR_BIN"),
            tmux_bin: non_empty_env("MAEH_TMUX_BIN"),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackendSettings {
    pub requested: BackendKind,
    pub selected: BackendKind,
    pub herdr_bin: String,
    pub tmux_bin: String,
}

impl BackendSettings {
    pub fn resolve(
        config_backend: BackendKind,
        config_herdr_bin: &str,
        config_tmux_bin: &str,
        env: &BackendEnv,
    ) -> Self {
        let requested = env.backend_override.unwrap_or(config_backend);
        let herdr_bin = match &env.herdr_bin {
            Some(bin) => bin.clone(),
            None => config_herdr_bin.to_string(),
        };
        let tmux_bin = match &env.tmux_bin {
            Some(bin) => bin.clone(),
            None => config_tmux_bin.to_string(),
        };
        Self {
            requested,
            selected: requested.resolve(env),
            herdr_bin,
            tmux_bin,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env: BTreeMap<String, String>,
}

impl CommandSpec {
    pub fn new(program: &str, args: &[&str]) -> Self {
        Self {
            program: program.to_string(),
            args: args.iter().map(|arg| (*arg).to_string()).collect(),
            cwd: None,
            env: BTreeMap::new(),
        }
    }

    pub fn argv(&self) -> Vec<String> {
        let mut argv = vec![self.program.clone()];
        argv.extend(self.args.clone());
        argv
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: i32,
}

pub trait CommandRunner {
    fn run(&mut self, spec: &CommandSpec) -> Result<CommandOutput, BackendError>;
}

#[derive(Default)]
pub struct RealRunner;

impl CommandRunner for RealRunner {
    fn run(&mut self, spec: &CommandSpec) -> Result<CommandOutput, BackendError> {
        let mut command = Command::new(&spec.program);
        command.args(&spec.args);
        if let Some(cwd) = &spec.cwd {
            command.current_dir(cwd);
        }
        command.envs(&spec.env);
        let output = command.output()?;
        Ok(CommandOutput {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            status: output.status.code().unwrap_or(1),
        })
    }
}

pub type SlotState = BTreeMap<String, BTreeMap<String, String>>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackendSlot {
    pub backend: BackendKind,
    pub slot: String,
    pub task_url: String,
    pub status: String,
    pub snooze_until: String,
    pub age_secs: u64,
    pub name: String,
    pub worktree: String,
    pub primary_pane: String,
    pub critic_pane: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationKind {
    Read,
    Mutate,
}

impl fmt::Display for OperationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Read => "read",
            Self::Mutate => "mutate",
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationPlan {
    pub kind: OperationKind,
    pub action: String,
    pub target: String,
    pub detail: String,
    pub command: Option<CommandSpec>,
}

impl OperationPlan {
    pub fn read(action: &str, target: &str, detail: String, command: Option<CommandSpec>) -> Self {
        Self {
            kind: OperationKind::Read,
            action: action.to_string(),
            target: target.to_string(),
            detail,
            command,
        }
    }

    pub fn mutate(action: &str, target: &str, detail: String) -> Self {
        Self {
            kind: OperationKind::Mutate,
            action: action.to_string(),
            target: target.to_string(),
            detail,
            command: None,
        }
    }
}

pub trait BackendAdapter {
    fn kind(&self) -> BackendKind;
    fn discovery_spec(&self) -> CommandSpec;
    fn parse_discovery(
        &self,
        stdout: &str,
        expected_state: &SlotState,
        now_epoch: u64,
    ) -> Result<Vec<BackendSlot>, BackendError>;
}

pub struct TmuxBackend {
    bin: String,
}

impl TmuxBackend {
    pub fn new(bin: String) -> Self {
        Self { bin }
    }
}

impl BackendAdapter for TmuxBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::Tmux
    }

    fn discovery_spec(&self) -> CommandSpec {
        CommandSpec::new(
            &self.bin,
            &[
                "list-windows",
                "-a",
                "-F",
                "#{session_name}:#{window_index}\u{1f}#{window_activity}\u{1f}#{window_name}\u{1f}#{@hmph_task}\u{1f}#{@hmph_status}\u{1f}#{@hmph_snooze_until}\u{1f}#{pane_current_path}",
            ],
        )
    }

    fn parse_discovery(
        &self,
        stdout: &str,
        _expected_state: &SlotState,
        now_epoch: u64,
    ) -> Result<Vec<BackendSlot>, BackendError> {
        let mut slots = Vec::new();
        for line in stdout.lines().filter(|line| !line.trim().is_empty()) {
            let fields = line.split('\u{1f}').collect::<Vec<_>>();
            if fields.len() != 7 {
                return Err(BackendError::Parse(format!(
                    "tmux discovery row has {} fields",
                    fields.len()
                )));
            }
            let task_url = fields[3].to_string();
            if task_url.is_empty() {
                continue;
            }
            let activity = fields[1].parse::<u64>().unwrap_or(now_epoch);
            slots.push(BackendSlot {
                backend: BackendKind::Tmux,
                slot: fields[0].to_string(),
                task_url,
                status: defaulted(fields[4], "none"),
                snooze_until: defaulted(fields[5], "0"),
                age_secs: now_epoch.saturating_sub(activity),
                name: fields[2].to_string(),
                worktree: fields[6].to_string(),
                primary_pane: format!("{}.1", fields[0]),
                critic_pane: format!("{}.2", fields[0]),
            });
        }
        Ok(slots)
    }
}

pub struct HerdrBackend {
    bin: String,
}

impl HerdrBackend {
    pub fn new(bin: String) -> Self {
        Self { bin }
    }
}

impl BackendAdapter for HerdrBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::Herdr
    }

    fn discovery_spec(&self) -> CommandSpec {
        CommandSpec::new(&self.bin, &["api", "snapshot"])
    }

    fn parse_discovery(
        &self,
        stdout: &str,
        expected_state: &SlotState,
        now_epoch: u64,
    ) -> Result<Vec<BackendSlot>, BackendError> {
        let payload: Value = serde_json::from_str(stdout)?;
        let snapshot = herdr_snapshot(&payload);
        let workspaces = match snapshot.get("workspaces") {
            Some(value) => match value.as_array() {
                Some(workspaces) => workspaces,
                None => {
                    return Err(BackendError::Parse(
                        "herdr snapshot missing workspaces".to_string(),
                    ))
                }
            },
            None => {
                return Err(BackendError::Parse(
                    "herdr snapshot missing workspaces".to_string(),
                ))
            }
        };
        let panes = match snapshot.get("panes") {
            Some(value) => match value.as_array() {
                Some(panes) => panes.clone(),
                None => Vec::new(),
            },
            None => Vec::new(),
        };
        let mut slots = Vec::new();
        for workspace in workspaces {
            let slot = value_str(workspace, "workspace_id");
            if slot.is_empty() {
                continue;
            }
            let Some(state) = expected_state.get(&slot) else {
                continue;
            };
            let task_url = state.get("task_url").cloned().unwrap_or_default();
            if task_url.is_empty() {
                continue;
            }
            let workspace_panes = panes_for_workspace(&panes, &slot);
            let worktree = herdr_worktree(workspace, state);
            let last_activity = match state.get("last_activity_epoch") {
                Some(value) => value.parse::<u64>().unwrap_or(now_epoch),
                None => now_epoch,
            };
            slots.push(BackendSlot {
                backend: BackendKind::Herdr,
                slot: slot.clone(),
                task_url,
                status: match state.get("status") {
                    Some(status) => status.clone(),
                    None => "none".to_string(),
                },
                snooze_until: match state.get("snooze_until") {
                    Some(snooze_until) => snooze_until.clone(),
                    None => "0".to_string(),
                },
                age_secs: now_epoch.saturating_sub(last_activity),
                name: value_str(workspace, "label"),
                worktree,
                primary_pane: match state.get("primary_pane") {
                    Some(primary_pane) => primary_pane.clone(),
                    None => first_agent_pane(&workspace_panes, "primary"),
                },
                critic_pane: match state.get("critic_pane") {
                    Some(critic_pane) => critic_pane.clone(),
                    None => first_agent_pane(&workspace_panes, "critic"),
                },
            });
        }
        Ok(slots)
    }
}

pub struct ReconciliationService<'a> {
    adapter: &'a dyn BackendAdapter,
}

impl<'a> ReconciliationService<'a> {
    pub fn new(adapter: &'a dyn BackendAdapter) -> Self {
        Self { adapter }
    }

    pub fn discovery_plan(&self) -> Vec<OperationPlan> {
        let target = self.adapter.kind().to_string();
        vec![OperationPlan::read(
            "discover",
            &target,
            "read backend state through adapter; no mutations".to_string(),
            Some(self.adapter.discovery_spec()),
        )]
    }

    pub fn discover_with_runner(
        &self,
        runner: &mut dyn CommandRunner,
        expected: &SlotState,
        now_epoch: u64,
    ) -> Result<Vec<BackendSlot>, BackendError> {
        let spec = self.adapter.discovery_spec();
        let output = runner.run(&spec)?;
        if output.status != 0 {
            return Err(BackendError::CommandFailed {
                program: spec.program,
                status: output.status,
            });
        }
        self.adapter
            .parse_discovery(&output.stdout, expected, now_epoch)
    }

    pub fn reconcile(
        &self,
        expected: &SlotState,
        discovered: &[BackendSlot],
    ) -> Vec<OperationPlan> {
        let mut operations = Vec::new();
        let discovered_by_slot = discovered
            .iter()
            .map(|slot| (slot.slot.as_str(), slot))
            .collect::<BTreeMap<_, _>>();
        for (slot, expected_state) in expected {
            let expected_url = expected_state.get("task_url").cloned().unwrap_or_default();
            if expected_url.is_empty() {
                continue;
            }
            match discovered_by_slot.get(slot.as_str()) {
                Some(live) if live.task_url == expected_url => operations.push(OperationPlan::read(
                    "ok",
                    slot,
                    format!(
                        "{} status={} age={}s",
                        live.task_url, live.status, live.age_secs
                    ),
                    None,
                )),
                Some(live) => operations.push(OperationPlan::mutate(
                    "metadata-drift",
                    slot,
                    format!(
                        "local task_url={} live task_url={}; dry-run only",
                        expected_url, live.task_url
                    ),
                )),
                None => operations.push(OperationPlan::mutate(
                    "missing-live-slot",
                    slot,
                    format!(
                        "local state tracks {}; dry-run action is delete local slot or respawn explicitly",
                        expected_url
                    ),
                )),
            }
        }
        for live in discovered {
            if !expected.contains_key(&live.slot) {
                operations.push(OperationPlan::mutate(
                    "untracked-live-slot",
                    &live.slot,
                    format!(
                        "backend has managed task {}; dry-run action is import into state service",
                        live.task_url
                    ),
                ));
            }
        }
        if operations.is_empty() {
            let target = self.adapter.kind().to_string();
            operations.push(OperationPlan::read(
                "ok",
                &target,
                "no managed slots found".to_string(),
                None,
            ));
        }
        operations
    }
}

pub fn adapter_for(settings: &BackendSettings) -> Box<dyn BackendAdapter> {
    match settings.selected {
        BackendKind::Auto => unreachable!("selected backend is resolved"),
        BackendKind::Herdr => Box::new(HerdrBackend::new(settings.herdr_bin.clone())),
        BackendKind::Tmux => Box::new(TmuxBackend::new(settings.tmux_bin.clone())),
    }
}

pub fn print_slots(slots: &[BackendSlot]) {
    for slot in slots {
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            slot.slot,
            slot.task_url,
            slot.status,
            slot.snooze_until,
            slot.age_secs,
            slot.name,
            slot.worktree,
            slot.primary_pane,
            slot.critic_pane
        );
    }
}

pub fn print_operations(operations: &[OperationPlan]) {
    for operation in operations {
        println!(
            "{}\t{}\t{}\t{}",
            operation.kind, operation.action, operation.target, operation.detail
        );
        if let Some(command) = &operation.command {
            println!(
                "  argv: {}",
                serde_json::to_string(&command.argv()).unwrap()
            );
            if let Some(cwd) = &command.cwd {
                println!("  cwd: {}", cwd.display());
            }
            for (key, value) in &command.env {
                println!("  env: {key}={value}");
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum BackendError {
    #[error("io: {0}")]
    Io(#[from] io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid backend: {0}")]
    InvalidBackend(String),
    #[error("backend command failed: {program} exited {status}")]
    CommandFailed { program: String, status: i32 },
    #[error("backend parse: {0}")]
    Parse(String),
}

fn non_empty_env(name: &str) -> Option<String> {
    match std::env::var(name) {
        Ok(value) if !value.is_empty() => Some(value),
        _ => None,
    }
}

fn defaulted(value: &str, fallback: &str) -> String {
    if value.is_empty() {
        fallback.to_string()
    } else {
        value.to_string()
    }
}

fn value_str(value: &Value, key: &str) -> String {
    match value.get(key) {
        Some(value) => match value.as_str() {
            Some(value) => value.to_string(),
            None => String::new(),
        },
        None => String::new(),
    }
}

fn herdr_snapshot(payload: &Value) -> &Value {
    let result_snapshot = match payload.get("result") {
        Some(result) => result.get("snapshot"),
        None => None,
    };
    match result_snapshot {
        Some(snapshot) => snapshot,
        None => match payload.get("snapshot") {
            Some(snapshot) => snapshot,
            None => payload,
        },
    }
}

fn panes_for_workspace<'a>(panes: &'a [Value], slot: &str) -> Vec<&'a Value> {
    let mut workspace_panes = Vec::new();
    for pane in panes {
        if value_str(pane, "workspace_id") == slot {
            workspace_panes.push(pane);
        }
    }
    workspace_panes
}

fn herdr_worktree(workspace: &Value, state: &BTreeMap<String, String>) -> String {
    let checkout_path = match workspace.get("worktree") {
        Some(worktree) => match worktree.get("checkout_path") {
            Some(checkout_path) => checkout_path.as_str(),
            None => None,
        },
        None => None,
    };
    match checkout_path {
        Some(checkout_path) => checkout_path.to_string(),
        None => match state.get("worktree") {
            Some(worktree) => worktree.clone(),
            None => String::new(),
        },
    }
}

fn first_agent_pane(panes: &[&Value], name: &str) -> String {
    for pane in panes {
        if value_str(pane, "agent").contains(name) {
            return value_str(pane, "pane_id");
        }
    }
    String::new()
}
