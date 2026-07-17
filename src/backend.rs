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
    pub tmux_session: Option<String>,
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
            tmux_session: non_empty_env("MAEH_TMUX_SESSION"),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BackendSettings {
    pub requested: BackendKind,
    pub selected: BackendKind,
    pub herdr_bin: String,
    pub tmux_bin: String,
    pub tmux_session: String,
}

impl BackendSettings {
    pub fn resolve(
        config_backend: BackendKind,
        config_herdr_bin: &str,
        config_tmux_bin: &str,
        config_tmux_session: &str,
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
        let tmux_session = match &env.tmux_session {
            Some(session) => session.clone(),
            None => config_tmux_session.to_string(),
        };
        Self {
            requested,
            selected: requested.resolve(env),
            herdr_bin,
            tmux_bin,
            tmux_session,
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

    pub fn from_args(program: &str, args: Vec<String>) -> Self {
        Self {
            program: program.to_string(),
            args,
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

    pub fn mutate_command(
        action: &str,
        target: &str,
        detail: String,
        command: CommandSpec,
    ) -> Self {
        Self {
            kind: OperationKind::Mutate,
            action: action.to_string(),
            target: target.to_string(),
            detail,
            command: Some(command),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LayoutOptions {
    pub include_editor: bool,
    pub focus: bool,
}

impl Default for LayoutOptions {
    fn default() -> Self {
        Self {
            include_editor: true,
            focus: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorktreeRequest {
    pub slot: String,
    pub repo: PathBuf,
    pub branch: String,
    pub base: String,
    pub path: PathBuf,
    pub label: String,
    pub create: bool,
    pub layout: LayoutOptions,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpawnRequest {
    pub worktree: WorktreeRequest,
    pub task_url: String,
    pub primary_agent_cmd: Vec<String>,
    pub critic_agent_cmd: Vec<String>,
    pub editor_cmd: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorktreeRecord {
    pub backend: BackendKind,
    pub slot: String,
    pub workspace_id: String,
    pub window_id: String,
    pub worktree: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpawnRecord {
    pub worktree: WorktreeRecord,
    pub primary_pane: String,
    pub critic_pane: String,
    pub editor_pane: String,
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
    fn worktree_plan(&self, request: &WorktreeRequest) -> Vec<OperationPlan>;
    fn spawn_plan(&self, request: &SpawnRequest) -> Vec<OperationPlan>;
    fn execute_worktree(
        &self,
        runner: &mut dyn CommandRunner,
        request: &WorktreeRequest,
    ) -> Result<WorktreeRecord, BackendError>;
    fn execute_spawn(
        &self,
        runner: &mut dyn CommandRunner,
        request: &SpawnRequest,
    ) -> Result<SpawnRecord, BackendError>;
    fn pane_read_spec(&self, target: &str) -> CommandSpec;
    fn delivery_specs(&self, target: &str, intent: &DeliveryIntent) -> Vec<CommandSpec>;
}

pub struct TmuxBackend {
    bin: String,
    session: String,
}

impl TmuxBackend {
    pub fn new(bin: String, session: String) -> Self {
        Self { bin, session }
    }

    fn new_window_spec(&self, request: &WorktreeRequest, command: Option<String>) -> CommandSpec {
        let mut args = vec!["new-window".to_string()];
        if !request.layout.focus {
            args.push("-d".to_string());
        }
        args.extend([
            "-P".to_string(),
            "-F".to_string(),
            "#{window_id}\t#{pane_id}".to_string(),
        ]);
        if !self.session.is_empty() {
            args.extend(["-t".to_string(), self.session.clone()]);
        }
        args.extend([
            "-n".to_string(),
            request.label.clone(),
            "-c".to_string(),
            display_path(&request.path),
        ]);
        if let Some(command) = command {
            args.push(command);
        }
        CommandSpec::from_args(&self.bin, args)
    }

    fn split_spec(
        &self,
        target: &str,
        path: &str,
        direction: &str,
        command: String,
    ) -> CommandSpec {
        CommandSpec::from_args(
            &self.bin,
            vec![
                "split-window".to_string(),
                "-d".to_string(),
                "-P".to_string(),
                "-F".to_string(),
                "#{pane_id}".to_string(),
                "-t".to_string(),
                target.to_string(),
                "-c".to_string(),
                path.to_string(),
                direction.to_string(),
                command,
            ],
        )
    }

    fn git_worktree_spec(&self, request: &WorktreeRequest) -> CommandSpec {
        let mut spec = CommandSpec::from_args(
            "git",
            vec![
                "-C".to_string(),
                display_path(&request.repo),
                "worktree".to_string(),
                "add".to_string(),
                "-b".to_string(),
                request.branch.clone(),
                display_path(&request.path),
            ],
        );
        if !request.base.is_empty() {
            spec.args.push(request.base.clone());
        }
        spec
    }
}

impl BackendAdapter for TmuxBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::Tmux
    }

    fn discovery_spec(&self) -> CommandSpec {
        let sep = "\u{1f}";
        let mut args = vec![
            "list-windows".to_string(),
            "-a".to_string(),
            "-F".to_string(),
            format!(
                "#{{session_name}}:#{{window_index}}{sep}#{{window_activity}}{sep}#{{window_name}}{sep}#{{@hmph_task}}{sep}#{{@hmph_status}}{sep}#{{@hmph_snooze_until}}{sep}#{{pane_current_path}}"
            ),
        ];
        if !self.session.is_empty() {
            args.splice(1..1, ["-t".to_string(), self.session.clone()]);
        }
        CommandSpec::from_args(&self.bin, args)
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
                primary_pane: String::new(),
                critic_pane: String::new(),
            });
        }
        Ok(slots)
    }

    fn worktree_plan(&self, request: &WorktreeRequest) -> Vec<OperationPlan> {
        let mut operations = Vec::new();
        if request.create {
            operations.push(OperationPlan::mutate_command(
                "worktree-create",
                &request.slot,
                format!("create git worktree {}", display_path(&request.path)),
                self.git_worktree_spec(request),
            ));
        }
        operations.push(OperationPlan::mutate_command(
            "workspace-open",
            &request.slot,
            format!("open tmux window at {}", display_path(&request.path)),
            self.new_window_spec(request, None),
        ));
        operations
    }

    fn spawn_plan(&self, request: &SpawnRequest) -> Vec<OperationPlan> {
        let mut operations = Vec::new();
        if request.worktree.create {
            operations.push(OperationPlan::mutate_command(
                "worktree-create",
                &request.worktree.slot,
                format!(
                    "create git worktree {}",
                    display_path(&request.worktree.path)
                ),
                self.git_worktree_spec(&request.worktree),
            ));
        }
        let target = "$window";
        let path = display_path(&request.worktree.path);
        operations.push(OperationPlan::mutate_command(
            "primary-agent",
            &request.worktree.slot,
            "open tmux window and start primary agent".to_string(),
            self.new_window_spec(
                &request.worktree,
                Some(shell_command(&request.primary_agent_cmd)),
            ),
        ));
        if request.worktree.layout.include_editor && !request.editor_cmd.is_empty() {
            operations.push(OperationPlan::mutate_command(
                "editor-pane",
                &request.worktree.slot,
                "start editor pane".to_string(),
                self.split_spec(target, &path, "-h", shell_command(&request.editor_cmd)),
            ));
        }
        operations.push(OperationPlan::mutate_command(
            "critic-agent",
            &request.worktree.slot,
            "start critic agent".to_string(),
            self.split_spec(
                target,
                &path,
                "-v",
                shell_command(&request.critic_agent_cmd),
            ),
        ));
        operations
    }

    fn execute_worktree(
        &self,
        runner: &mut dyn CommandRunner,
        request: &WorktreeRequest,
    ) -> Result<WorktreeRecord, BackendError> {
        if request.create {
            run_ok(runner, &self.git_worktree_spec(request))?;
        }
        let output = run_ok(runner, &self.new_window_spec(request, None))?;
        let (window_id, _) = parse_tmux_window_pane(&output.stdout)?;
        Ok(WorktreeRecord {
            backend: BackendKind::Tmux,
            slot: request.slot.clone(),
            workspace_id: window_id.clone(),
            window_id,
            worktree: display_path(&request.path),
        })
    }

    fn execute_spawn(
        &self,
        runner: &mut dyn CommandRunner,
        request: &SpawnRequest,
    ) -> Result<SpawnRecord, BackendError> {
        if request.worktree.create {
            run_ok(runner, &self.git_worktree_spec(&request.worktree))?;
        }
        let primary_command = shell_command(&request.primary_agent_cmd);
        let window_spec = self.new_window_spec(&request.worktree, Some(primary_command));
        let output = run_ok(runner, &window_spec)?;
        let (window_id, primary_pane) = parse_tmux_window_pane(&output.stdout)?;
        let path = display_path(&request.worktree.path);
        let editor_pane =
            if request.worktree.layout.include_editor && !request.editor_cmd.is_empty() {
                let editor_spec =
                    self.split_spec(&window_id, &path, "-h", shell_command(&request.editor_cmd));
                run_ok(runner, &editor_spec)?.stdout.trim().to_string()
            } else {
                String::new()
            };
        let critic_spec = self.split_spec(
            &window_id,
            &path,
            "-v",
            shell_command(&request.critic_agent_cmd),
        );
        let critic_pane = run_ok(runner, &critic_spec)?.stdout.trim().to_string();
        Ok(SpawnRecord {
            worktree: WorktreeRecord {
                backend: BackendKind::Tmux,
                slot: request.worktree.slot.clone(),
                workspace_id: window_id.clone(),
                window_id,
                worktree: path,
            },
            primary_pane,
            critic_pane,
            editor_pane,
        })
    }

    fn pane_read_spec(&self, target: &str) -> CommandSpec {
        CommandSpec::from_args(
            &self.bin,
            vec![
                "capture-pane".to_string(),
                "-p".to_string(),
                "-t".to_string(),
                target.to_string(),
                "-S".to_string(),
                "-80".to_string(),
            ],
        )
    }

    fn delivery_specs(&self, target: &str, intent: &DeliveryIntent) -> Vec<CommandSpec> {
        match intent {
            DeliveryIntent::Noop { .. } => Vec::new(),
            DeliveryIntent::SubmitQueued { text }
            | DeliveryIntent::AnswerBlocker { response: text, .. } => {
                let mut specs = Vec::new();
                if !text.is_empty() {
                    specs.push(CommandSpec::from_args(
                        &self.bin,
                        vec![
                            "send-keys".to_string(),
                            "-t".to_string(),
                            target.to_string(),
                            "-l".to_string(),
                            text.clone(),
                        ],
                    ));
                }
                specs.push(CommandSpec::from_args(
                    &self.bin,
                    vec![
                        "send-keys".to_string(),
                        "-t".to_string(),
                        target.to_string(),
                        "Enter".to_string(),
                    ],
                ));
                specs
            }
        }
    }
}

pub struct HerdrBackend {
    bin: String,
}

impl HerdrBackend {
    pub fn new(bin: String) -> Self {
        Self { bin }
    }

    fn worktree_spec(&self, request: &WorktreeRequest) -> CommandSpec {
        let mut args = vec![
            "worktree".to_string(),
            if request.create { "create" } else { "open" }.to_string(),
            "--cwd".to_string(),
            display_path(&request.repo),
        ];
        if request.create {
            args.extend(["--branch".to_string(), request.branch.clone()]);
            if !request.base.is_empty() {
                args.extend(["--base".to_string(), request.base.clone()]);
            }
        }
        if !request.path.as_os_str().is_empty() {
            args.extend(["--path".to_string(), display_path(&request.path)]);
        } else if !request.branch.is_empty() {
            args.extend(["--branch".to_string(), request.branch.clone()]);
        }
        if !request.label.is_empty() {
            args.extend(["--label".to_string(), request.label.clone()]);
        }
        if request.layout.focus {
            args.push("--focus".to_string());
        } else {
            args.push("--no-focus".to_string());
        }
        args.push("--json".to_string());
        CommandSpec::from_args(&self.bin, args)
    }

    fn agent_start_spec(
        &self,
        name: &str,
        path: &str,
        workspace: &str,
        split: &str,
        command: &[String],
    ) -> CommandSpec {
        let mut args = vec![
            "agent".to_string(),
            "start".to_string(),
            name.to_string(),
            "--cwd".to_string(),
            path.to_string(),
            "--workspace".to_string(),
            workspace.to_string(),
            "--split".to_string(),
            split.to_string(),
            "--no-focus".to_string(),
            "--".to_string(),
        ];
        args.extend(command.iter().cloned());
        CommandSpec::from_args(&self.bin, args)
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

    fn worktree_plan(&self, request: &WorktreeRequest) -> Vec<OperationPlan> {
        vec![OperationPlan::mutate_command(
            if request.create {
                "worktree-create"
            } else {
                "worktree-open"
            },
            &request.slot,
            format!(
                "{} Herdr worktree/workspace",
                if request.create { "create" } else { "open" }
            ),
            self.worktree_spec(request),
        )]
    }

    fn spawn_plan(&self, request: &SpawnRequest) -> Vec<OperationPlan> {
        let mut operations = self.worktree_plan(&request.worktree);
        let path = if request.worktree.path.as_os_str().is_empty() {
            "$worktree".to_string()
        } else {
            display_path(&request.worktree.path)
        };
        let workspace = "$workspace";
        if request.worktree.layout.include_editor && !request.editor_cmd.is_empty() {
            operations.push(OperationPlan::mutate_command(
                "editor-pane",
                &request.worktree.slot,
                "start editor pane".to_string(),
                self.agent_start_spec("editor", &path, workspace, "right", &request.editor_cmd),
            ));
        }
        operations.push(OperationPlan::mutate_command(
            "primary-agent",
            &request.worktree.slot,
            "start primary agent".to_string(),
            self.agent_start_spec(
                "primary",
                &path,
                workspace,
                "right",
                &request.primary_agent_cmd,
            ),
        ));
        operations.push(OperationPlan::mutate_command(
            "critic-agent",
            &request.worktree.slot,
            "start critic agent".to_string(),
            self.agent_start_spec(
                "critic",
                &path,
                workspace,
                "down",
                &request.critic_agent_cmd,
            ),
        ));
        operations
    }

    fn execute_worktree(
        &self,
        runner: &mut dyn CommandRunner,
        request: &WorktreeRequest,
    ) -> Result<WorktreeRecord, BackendError> {
        let output = run_ok(runner, &self.worktree_spec(request))?;
        let payload: Value = serde_json::from_str(&output.stdout)?;
        let workspace_id = find_string_key(&payload, &["workspace_id", "open_workspace_id"])
            .ok_or_else(|| {
                BackendError::Parse("herdr worktree output missing workspace id".to_string())
            })?;
        let worktree = find_string_key(&payload, &["path", "checkout_path"])
            .unwrap_or_else(|| display_path(&request.path));
        Ok(WorktreeRecord {
            backend: BackendKind::Herdr,
            slot: request.slot.clone(),
            workspace_id,
            window_id: String::new(),
            worktree,
        })
    }

    fn execute_spawn(
        &self,
        runner: &mut dyn CommandRunner,
        request: &SpawnRequest,
    ) -> Result<SpawnRecord, BackendError> {
        let worktree = self.execute_worktree(runner, &request.worktree)?;
        let editor_pane =
            if request.worktree.layout.include_editor && !request.editor_cmd.is_empty() {
                let editor_spec = self.agent_start_spec(
                    "editor",
                    &worktree.worktree,
                    &worktree.workspace_id,
                    "right",
                    &request.editor_cmd,
                );
                let output = run_ok(runner, &editor_spec)?;
                parse_pane_id(&output.stdout).unwrap_or_default()
            } else {
                String::new()
            };
        let primary_spec = self.agent_start_spec(
            "primary",
            &worktree.worktree,
            &worktree.workspace_id,
            "right",
            &request.primary_agent_cmd,
        );
        let primary_output = run_ok(runner, &primary_spec)?;
        let critic_spec = self.agent_start_spec(
            "critic",
            &worktree.worktree,
            &worktree.workspace_id,
            "down",
            &request.critic_agent_cmd,
        );
        let critic_output = run_ok(runner, &critic_spec)?;
        Ok(SpawnRecord {
            worktree,
            primary_pane: parse_pane_id(&primary_output.stdout).unwrap_or_default(),
            critic_pane: parse_pane_id(&critic_output.stdout).unwrap_or_default(),
            editor_pane,
        })
    }

    fn pane_read_spec(&self, target: &str) -> CommandSpec {
        CommandSpec::from_args(
            &self.bin,
            vec![
                "agent".to_string(),
                "read".to_string(),
                target.to_string(),
                "--source".to_string(),
                "recent-unwrapped".to_string(),
                "--lines".to_string(),
                "80".to_string(),
                "--format".to_string(),
                "text".to_string(),
            ],
        )
    }

    fn delivery_specs(&self, target: &str, intent: &DeliveryIntent) -> Vec<CommandSpec> {
        match intent {
            DeliveryIntent::Noop { .. } => Vec::new(),
            DeliveryIntent::SubmitQueued { text }
            | DeliveryIntent::AnswerBlocker { response: text, .. } => {
                let mut specs = Vec::new();
                if !text.is_empty() {
                    specs.push(CommandSpec::from_args(
                        &self.bin,
                        vec![
                            "agent".to_string(),
                            "send".to_string(),
                            target.to_string(),
                            text.clone(),
                        ],
                    ));
                }
                specs.push(CommandSpec::from_args(
                    &self.bin,
                    vec![
                        "pane".to_string(),
                        "send-keys".to_string(),
                        target.to_string(),
                        "Enter".to_string(),
                    ],
                ));
                specs
            }
        }
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
        BackendKind::Tmux => Box::new(TmuxBackend::new(
            settings.tmux_bin.clone(),
            settings.tmux_session.clone(),
        )),
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HarnessBlocker {
    Trust,
    Update,
    Continue,
}

impl fmt::Display for HarnessBlocker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Trust => "trust",
            Self::Update => "update",
            Self::Continue => "continue",
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeliveryIntent {
    SubmitQueued {
        text: String,
    },
    AnswerBlocker {
        blocker: HarnessBlocker,
        response: String,
    },
    Noop {
        reason: String,
    },
}

impl DeliveryIntent {
    pub fn action(&self) -> String {
        match self {
            Self::SubmitQueued { .. } => "submit-queued".to_string(),
            Self::AnswerBlocker { blocker, .. } => format!("unblock-{blocker}"),
            Self::Noop { .. } => "noop".to_string(),
        }
    }

    pub fn detail(&self) -> String {
        match self {
            Self::SubmitQueued { text } => {
                format!("send {} chars plus explicit Enter", text.chars().count())
            }
            Self::AnswerBlocker { blocker, response } => format!(
                "answer {blocker} blocker with {} plus explicit Enter",
                if response.is_empty() {
                    "Enter"
                } else {
                    response
                }
            ),
            Self::Noop { reason } => reason.clone(),
        }
    }
}

pub fn delivery_intent(pane_text: &str, queued_prompt: &str) -> DeliveryIntent {
    if let Some((blocker, response)) = detect_blocker(pane_text) {
        return DeliveryIntent::AnswerBlocker { blocker, response };
    }
    if queued_prompt.trim().is_empty() {
        return DeliveryIntent::Noop {
            reason: "no queued prompt".to_string(),
        };
    }
    if pane_has_input_prompt(pane_text) {
        return DeliveryIntent::SubmitQueued {
            text: queued_prompt.to_string(),
        };
    }
    DeliveryIntent::Noop {
        reason: "pane busy or unknown".to_string(),
    }
}

pub fn delivery_plan(
    adapter: &dyn BackendAdapter,
    target: &str,
    pane_text: &str,
    queued_prompt: &str,
) -> Vec<OperationPlan> {
    let intent = delivery_intent(pane_text, queued_prompt);
    let mut operations = Vec::new();
    let has_text_event = match &intent {
        DeliveryIntent::SubmitQueued { text } => !text.is_empty(),
        DeliveryIntent::AnswerBlocker { response, .. } => !response.is_empty(),
        DeliveryIntent::Noop { .. } => false,
    };
    let specs = adapter.delivery_specs(target, &intent);
    if specs.is_empty() {
        operations.push(OperationPlan::read(
            &intent.action(),
            target,
            intent.detail(),
            None,
        ));
    } else {
        for (index, spec) in specs.into_iter().enumerate() {
            let action = if index == 0 && has_text_event {
                "send-text"
            } else {
                "submit-enter"
            };
            operations.push(OperationPlan::mutate_command(
                action,
                target,
                intent.detail(),
                spec,
            ));
        }
    }
    operations
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptVerification {
    pub changed: bool,
    pub submitted: bool,
    pub prompt_head: String,
}

pub fn pane_text_from_read_output(kind: BackendKind, stdout: &str) -> Result<String, BackendError> {
    match kind {
        BackendKind::Herdr => {
            let payload: Value = serde_json::from_str(stdout)?;
            find_string_key(&payload, &["text"]).ok_or_else(|| {
                BackendError::Parse("herdr pane read output missing text".to_string())
            })
        }
        BackendKind::Tmux | BackendKind::Auto => Ok(stdout.to_string()),
    }
}

pub fn verify_prompt_execution(
    before: &str,
    after: &str,
    prompt: &str,
) -> Result<PromptVerification, BackendError> {
    let changed = before != after;
    let prompt_head = first_prompt_line(prompt);
    let last_line = after
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("");
    let still_pending = !prompt_head.is_empty() && last_line.contains(&prompt_head);
    let submitted = changed && !still_pending;
    if !submitted {
        return Err(BackendError::Verify("prompt did not execute".to_string()));
    }
    Ok(PromptVerification {
        changed,
        submitted,
        prompt_head,
    })
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
    #[error("verify: {0}")]
    Verify(String),
}

fn run_ok(
    runner: &mut dyn CommandRunner,
    spec: &CommandSpec,
) -> Result<CommandOutput, BackendError> {
    let output = runner.run(spec)?;
    if output.status != 0 {
        return Err(BackendError::CommandFailed {
            program: spec.program.clone(),
            status: output.status,
        });
    }
    Ok(output)
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

fn display_path(path: &std::path::Path) -> String {
    path.display().to_string()
}

fn shell_command(argv: &[String]) -> String {
    argv.join(" ")
}

fn parse_tmux_window_pane(stdout: &str) -> Result<(String, String), BackendError> {
    let line = stdout.trim();
    let fields = line.split('\t').collect::<Vec<_>>();
    if fields.len() == 2 {
        return Ok((fields[0].to_string(), fields[1].to_string()));
    }
    Err(BackendError::Parse(
        "tmux window output missing window/pane ids".to_string(),
    ))
}

fn parse_pane_id(stdout: &str) -> Option<String> {
    let payload = serde_json::from_str::<Value>(stdout).ok()?;
    find_string_key(&payload, &["pane_id", "id", "terminal_id"])
}

fn find_string_key(value: &Value, keys: &[&str]) -> Option<String> {
    match value {
        Value::Object(map) => {
            for key in keys {
                if let Some(value) = map.get(*key).and_then(Value::as_str) {
                    return Some(value.to_string());
                }
            }
            for value in map.values() {
                if let Some(found) = find_string_key(value, keys) {
                    return Some(found);
                }
            }
            None
        }
        Value::Array(values) => {
            for value in values {
                if let Some(found) = find_string_key(value, keys) {
                    return Some(found);
                }
            }
            None
        }
        _ => None,
    }
}

fn detect_blocker(pane_text: &str) -> Option<(HarnessBlocker, String)> {
    let lower = pane_text.to_lowercase();
    if lower.contains("do you trust") || (lower.contains("trust") && lower.contains("folder")) {
        return Some((HarnessBlocker::Trust, "1".to_string()));
    }
    if lower.contains("update") && (lower.contains("available") || lower.contains("new version")) {
        return Some((HarnessBlocker::Update, "n".to_string()));
    }
    if lower.contains("press enter to continue") || lower.contains("continue?") {
        return Some((HarnessBlocker::Continue, String::new()));
    }
    None
}

fn pane_has_input_prompt(pane_text: &str) -> bool {
    let Some(line) = pane_text.lines().rev().find(|line| !line.trim().is_empty()) else {
        return false;
    };
    let trimmed = line.trim_end();
    trimmed.ends_with('›')
        || trimmed.ends_with('>')
        || trimmed.ends_with('$')
        || trimmed.contains("Type something")
}

fn first_prompt_line(prompt: &str) -> String {
    prompt
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("")
        .trim()
        .chars()
        .take(80)
        .collect()
}
