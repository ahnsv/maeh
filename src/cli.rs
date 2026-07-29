use std::path::{Path, PathBuf};

use clap::{Args, Parser, Subcommand};

use crate::error::{MaehError, Result};

#[derive(Parser, Debug)]
#[command(
    name = "maeh",
    version,
    about = "Typed orchestration CLI for hmph and Herdr agents",
    disable_help_subcommand = true
)]
pub(crate) struct Cli {
    #[arg(
        long,
        value_name = "PATH",
        global = true,
        help = "Use alternate state directory"
    )]
    pub(crate) home: Option<PathBuf>,
    #[command(subcommand)]
    pub(crate) command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    #[command(about = "Create local state directories and config")]
    Init,
    #[command(about = "Inspect paths, effective config, and env exports")]
    Config(ConfigArgs),
    #[command(about = "Append or list orchestration JSONL spans")]
    Ledger(LedgerArgs),
    #[command(about = "Read and mutate local slot metadata")]
    State(StateArgs),
    #[command(about = "Store and read tracker board snapshots")]
    BoardCache(BoardCacheArgs),
    #[command(about = "Store and render compact task context")]
    Capsule(CapsuleArgs),
    #[command(about = "Render reusable agent prompts")]
    Prompt(PromptArgs),
    #[command(about = "Inspect and reconcile Herdr/tmux backend state")]
    Backend(BackendArgs),
    #[command(about = "Plan or open backend worktrees/workspaces")]
    Worktree(WorktreeArgs),
    #[command(about = "Register or spawn managed backend workspaces")]
    Workspace(WorkspaceArgs),
    #[command(about = "Plan or launch a worktree plus primary/critic agents")]
    Spawn(SpawnArgs),
    #[command(about = "Deliver prompts through backend adapters")]
    Agent(AgentArgs),
    #[command(about = "Plan or run queued prompt delivery")]
    Kickoff(KickoffArgs),
    #[command(about = "Verify prompt or slot execution evidence")]
    Verify(VerifyArgs),
    #[command(about = "List, inspect, classify, and mutate managed slots")]
    Slot(SlotArgs),
    #[command(about = "Cleanup-oriented wrappers for done slots")]
    Cleanup(CleanupArgs),
    #[command(about = "Stale-work wrappers for resume/snooze/block/nudge")]
    Revamp(RevampArgs),
    #[command(about = "Backend-aware slot and worktree reports")]
    Status(StatusArgs),
    #[command(about = "Check configured work/review caps")]
    Cap(CapArgs),
    #[command(about = "Print compact pool status")]
    Statusline,
    #[command(about = "Evaluate configured work-hour guard")]
    WorkHours,
    #[command(about = "Debug paths, config, backend, and env")]
    Doctor,
    #[command(about = "Validate config/state readability")]
    Selftest,
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args, Debug)]
pub(crate) struct ConfigArgs {
    #[command(subcommand)]
    command: Option<ConfigSubcommand>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum ConfigSubcommand {
    #[command(about = "Print the config.toml path for the active home")]
    Path,
    #[command(about = "Print the effective human-readable config")]
    Show,
    #[command(about = "Print shell-friendly MAEH_* key/value lines")]
    Emit,
    #[command(about = "Persist the default home in the XDG config")]
    SetHome(ConfigSetHomeArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args, Debug)]
pub(crate) struct ConfigSetHomeArgs {
    #[arg(value_name = "PATH", allow_hyphen_values = true)]
    path: Option<String>,
}

#[derive(Args, Debug)]
pub(crate) struct LedgerArgs {
    #[command(subcommand)]
    command: Option<LedgerSubcommand>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum LedgerSubcommand {
    #[command(about = "Append one span row to <home>/ledger/<loop>.jsonl")]
    Append(LedgerAppendArgs),
    #[command(about = "Print rows from a loop ledger file")]
    List(LedgerListArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args, Debug)]
pub(crate) struct LedgerAppendArgs {
    #[arg(long = "loop", value_name = "NAME", num_args = 0..=1, allow_hyphen_values = true)]
    loop_name: Option<Option<String>>,
    #[arg(long, value_name = "NAME", allow_hyphen_values = true)]
    event: Option<String>,
    #[arg(long, value_name = "VALUE", allow_hyphen_values = true)]
    target: Option<String>,
    #[arg(long, value_name = "JSON", allow_hyphen_values = true)]
    data: Option<String>,
}

#[derive(Args, Debug)]
pub(crate) struct LedgerListArgs {
    #[arg(long = "loop", value_name = "NAME", num_args = 0..=1, allow_hyphen_values = true)]
    loop_name: Option<Option<String>>,
}

#[derive(Args, Debug)]
pub(crate) struct StateArgs {
    #[command(subcommand)]
    command: Option<StateSubcommand>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum StateSubcommand {
    #[command(about = "Set a key/value on a slot")]
    Tag {
        #[arg(allow_hyphen_values = true)]
        slot: String,
        #[arg(allow_hyphen_values = true)]
        key: String,
        #[arg(allow_hyphen_values = true)]
        value: String,
    },
    #[command(about = "Remove one key from a slot")]
    Untag {
        #[arg(allow_hyphen_values = true)]
        slot: String,
        #[arg(allow_hyphen_values = true)]
        key: String,
    },
    #[command(about = "Print one slot value")]
    Get {
        #[arg(allow_hyphen_values = true)]
        slot: String,
        #[arg(allow_hyphen_values = true)]
        key: String,
    },
    #[command(about = "Print tab-separated slot summary rows")]
    List,
    #[command(about = "Shortcut for get <slot> worktree")]
    Worktree {
        #[arg(allow_hyphen_values = true)]
        slot: String,
    },
    #[command(about = "Remove the local slot record")]
    DeleteSlot {
        #[arg(allow_hyphen_values = true)]
        slot: String,
    },
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args, Debug)]
pub(crate) struct BoardCacheArgs {
    #[command(subcommand)]
    command: Option<BoardCacheSubcommand>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum BoardCacheSubcommand {
    #[command(about = "Read JSON from stdin and store it under a cache key")]
    Put(KeyArg),
    #[command(about = "Print cached JSON when it exists and is fresh")]
    Get(BoardCacheGetArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args, Debug)]
pub(crate) struct KeyArg {
    #[arg(long, value_name = "NAME", allow_hyphen_values = true)]
    key: Option<String>,
}

#[derive(Args, Debug)]
pub(crate) struct BoardCacheGetArgs {
    #[arg(long, value_name = "NAME", allow_hyphen_values = true)]
    key: Option<String>,
    #[arg(long)]
    stale: bool,
}

#[derive(Args, Debug)]
pub(crate) struct CapsuleArgs {
    #[command(subcommand)]
    command: Option<CapsuleSubcommand>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum CapsuleSubcommand {
    #[command(about = "Read JSON from stdin and cache it for a task URL")]
    Put(CapsuleUrlArgs),
    #[command(about = "Print cached capsule JSON")]
    Get(CapsuleUrlArgs),
    #[command(about = "Render cached capsule inside a prompt block")]
    Prompt(CapsuleUrlArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args, Debug)]
pub(crate) struct CapsuleUrlArgs {
    #[arg(allow_hyphen_values = true)]
    url: String,
    #[arg(long, value_name = "VALUE", allow_hyphen_values = true)]
    edited: Option<String>,
}

#[derive(Args, Debug)]
pub(crate) struct PromptArgs {
    #[command(subcommand)]
    command: Option<PromptSubcommand>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum PromptSubcommand {
    #[command(about = "Render the standard kickoff prompt for a tracker task")]
    Kickoff(PromptKickoffArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args, Debug)]
pub(crate) struct PromptKickoffArgs {
    #[arg(long, value_name = "URL", allow_hyphen_values = true)]
    url: Option<String>,
    #[arg(long = "capsule-file", value_name = "PATH", allow_hyphen_values = true)]
    capsule_file: Option<String>,
}

#[derive(Args, Debug)]
pub(crate) struct BackendArgs {
    #[arg(long, value_name = "PATH", global = true, allow_hyphen_values = true)]
    fixture: Option<String>,
    #[arg(long, global = true)]
    exec: bool,
    #[command(subcommand)]
    command: Option<BackendSubcommand>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum BackendSubcommand {
    #[command(about = "Print the backend discovery command without running it")]
    Plan,
    #[command(about = "Read backend state and print normalized slot rows")]
    Discover,
    #[command(about = "Compare backend state with local state and print operations")]
    Reconcile,
    #[command(about = "Print task-oriented slot rows")]
    ListTaskSlots,
    #[command(about = "Print locally tracked worktree rows")]
    ListWorktrees,
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args, Debug)]
pub(crate) struct WorktreeArgs {
    #[command(subcommand)]
    command: Option<WorktreeSubcommand>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum WorktreeSubcommand {
    #[command(about = "Print backend operations without mutating anything")]
    Plan(WorktreeOptions),
    #[command(about = "Execute worktree/workspace creation and persist local state")]
    Open(WorktreeOptions),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args, Debug)]
pub(crate) struct WorktreeOptions {
    #[arg(long, value_name = "SLOT", allow_hyphen_values = true)]
    slot: Option<String>,
    #[arg(long, value_name = "PATH", allow_hyphen_values = true)]
    repo: Option<String>,
    #[arg(long, value_name = "NAME", allow_hyphen_values = true)]
    branch: Option<String>,
    #[arg(long, value_name = "REF", allow_hyphen_values = true)]
    base: Option<String>,
    #[arg(long, value_name = "PATH", allow_hyphen_values = true)]
    path: Option<String>,
    #[arg(long, value_name = "NAME", allow_hyphen_values = true)]
    label: Option<String>,
    #[arg(long)]
    create: bool,
    #[command(flatten)]
    layout: LayoutArgs,
}

#[derive(Args, Debug, Default)]
pub(crate) struct LayoutArgs {
    #[arg(long = "with-editor")]
    with_editor: bool,
    #[arg(long = "no-editor")]
    no_editor: bool,
    #[arg(long, value_name = "BOOL", allow_hyphen_values = true)]
    editor: Option<String>,
    #[arg(long)]
    focus: bool,
    #[arg(long = "no-focus")]
    no_focus: bool,
}

#[derive(Args, Debug)]
pub(crate) struct WorkspaceArgs {
    #[command(subcommand)]
    command: Option<WorkspaceSubcommand>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum WorkspaceSubcommand {
    #[command(about = "Persist an existing workspace, panes, worktree, and metadata")]
    Register(WorkspaceRegisterArgs),
    #[command(about = "Plan or execute a managed workspace plus agent spawn")]
    Spawn(SlotSpawnOptions),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args, Debug)]
pub(crate) struct WorkspaceRegisterArgs {
    #[arg(long, value_name = "SLOT", allow_hyphen_values = true)]
    slot: String,
    #[arg(long, value_name = "ID", allow_hyphen_values = true)]
    workspace: String,
    #[arg(long, value_name = "PATH", allow_hyphen_values = true)]
    worktree: String,
    #[arg(long, value_name = "PATH", allow_hyphen_values = true)]
    repo: Option<String>,
    #[arg(long = "task-url", value_name = "URL", allow_hyphen_values = true)]
    task_url: Option<String>,
    #[arg(long = "primary-pane", value_name = "ID", allow_hyphen_values = true)]
    primary_pane: Option<String>,
    #[arg(long = "critic-pane", value_name = "ID", allow_hyphen_values = true)]
    critic_pane: Option<String>,
    #[arg(long = "editor-pane", value_name = "ID", allow_hyphen_values = true)]
    editor_pane: Option<String>,
    #[arg(long, value_name = "KIND", allow_hyphen_values = true)]
    backend: Option<String>,
    #[arg(long, value_name = "VALUE", allow_hyphen_values = true)]
    status: Option<String>,
}

#[derive(Args, Debug)]
pub(crate) struct SpawnArgs {
    #[command(subcommand)]
    command: Option<SpawnSubcommand>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum SpawnSubcommand {
    #[command(about = "Print backend operations without mutating anything")]
    Plan(SpawnOptions),
    #[command(about = "Execute worktree and agent startup, then persist local state")]
    Run(SpawnOptions),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args, Debug)]
pub(crate) struct SpawnOptions {
    #[arg(long = "task-url", value_name = "URL", allow_hyphen_values = true)]
    task_url: String,
    #[command(flatten)]
    worktree: WorktreeOptions,
    #[arg(long = "primary-cmd", value_name = "CMD", allow_hyphen_values = true)]
    primary_cmd: Option<String>,
    #[arg(long = "critic-cmd", value_name = "CMD", allow_hyphen_values = true)]
    critic_cmd: Option<String>,
    #[arg(long = "editor-cmd", value_name = "CMD", allow_hyphen_values = true)]
    editor_cmd: Option<String>,
}

#[derive(Args, Debug)]
pub(crate) struct SlotSpawnOptions {
    #[arg(long, value_name = "KIND", allow_hyphen_values = true)]
    backend: Option<String>,
    #[arg(long)]
    exec: bool,
    #[arg(long, value_name = "SLOT", allow_hyphen_values = true)]
    slot: Option<String>,
    #[arg(
        long,
        alias = "repo-root",
        value_name = "PATH",
        allow_hyphen_values = true
    )]
    repo: String,
    #[arg(long, value_name = "NAME", allow_hyphen_values = true)]
    branch: Option<String>,
    #[arg(long, value_name = "REF", allow_hyphen_values = true)]
    base: Option<String>,
    #[arg(
        long,
        alias = "worktree",
        value_name = "PATH",
        allow_hyphen_values = true
    )]
    path: String,
    #[arg(long, value_name = "NAME", allow_hyphen_values = true)]
    label: Option<String>,
    #[arg(long)]
    create: bool,
    #[arg(long = "open-existing")]
    open_existing: bool,
    #[arg(long = "task-url", value_name = "URL", allow_hyphen_values = true)]
    task_url: String,
    #[arg(long = "primary-cmd", value_name = "CMD", allow_hyphen_values = true)]
    primary_cmd: Option<String>,
    #[arg(long = "critic-cmd", value_name = "CMD", allow_hyphen_values = true)]
    critic_cmd: Option<String>,
    #[arg(long = "editor-cmd", value_name = "CMD", allow_hyphen_values = true)]
    editor_cmd: Option<String>,
    #[command(flatten)]
    layout: LayoutArgs,
}

#[derive(Args, Debug)]
pub(crate) struct AgentArgs {
    #[command(subcommand)]
    command: Option<AgentSubcommand>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum AgentSubcommand {
    #[command(about = "Plan or execute prompt delivery to a target pane or slot role")]
    Deliver(AgentDeliverArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args, Debug)]
pub(crate) struct AgentDeliverArgs {
    #[command(flatten)]
    deliver: DeliverArgs,
    #[arg(long)]
    exec: bool,
}

#[derive(Args, Debug, Default)]
pub(crate) struct DeliverArgs {
    #[arg(value_name = "TARGET", allow_hyphen_values = true)]
    positional_target: Option<String>,
    #[arg(value_name = "PROMPT", allow_hyphen_values = true)]
    positional_prompt: Option<String>,
    #[arg(long, value_name = "ID", allow_hyphen_values = true)]
    target: Option<String>,
    #[arg(long, value_name = "SLOT", allow_hyphen_values = true)]
    slot: Option<String>,
    #[arg(long, value_name = "ROLE", allow_hyphen_values = true)]
    role: Option<String>,
    #[arg(long, value_name = "TEXT", allow_hyphen_values = true)]
    prompt: Option<String>,
    #[arg(long = "prompt-file", value_name = "PATH", allow_hyphen_values = true)]
    prompt_file: Option<String>,
    #[arg(long = "pane-text", value_name = "TEXT", allow_hyphen_values = true)]
    pane_text: Option<String>,
    #[arg(long = "pane-file", value_name = "PATH", allow_hyphen_values = true)]
    pane_file: Option<String>,
}

#[derive(Args, Debug)]
pub(crate) struct KickoffArgs {
    #[command(subcommand)]
    command: Option<KickoffSubcommand>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum KickoffSubcommand {
    #[command(about = "Print prompt delivery operations without executing them")]
    Plan(DeliverArgs),
    #[command(about = "Execute prompt delivery operations")]
    Run(DeliverArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args, Debug)]
pub(crate) struct VerifyArgs {
    #[command(subcommand)]
    command: Option<VerifySubcommand>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum VerifySubcommand {
    #[command(about = "Compare before/after pane text against a prompt")]
    Prompt(VerifyPromptArgs),
    #[command(about = "Verify local slot has worktree, primary pane, and critic pane")]
    Slot(VerifySlotArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args, Debug)]
pub(crate) struct VerifyPromptArgs {
    #[arg(long, value_name = "TEXT", allow_hyphen_values = true)]
    before: Option<String>,
    #[arg(long = "before-file", value_name = "PATH", allow_hyphen_values = true)]
    before_file: Option<String>,
    #[arg(long, value_name = "TEXT", allow_hyphen_values = true)]
    after: Option<String>,
    #[arg(long = "after-file", value_name = "PATH", allow_hyphen_values = true)]
    after_file: Option<String>,
    #[arg(long, value_name = "TEXT", allow_hyphen_values = true)]
    prompt: Option<String>,
    #[arg(long = "prompt-file", value_name = "PATH", allow_hyphen_values = true)]
    prompt_file: Option<String>,
}

#[derive(Args, Debug, Default)]
pub(crate) struct SlotRefArgs {
    #[arg(value_name = "SLOT", allow_hyphen_values = true)]
    slot: Option<String>,
    #[arg(long = "slot", value_name = "SLOT", allow_hyphen_values = true)]
    slot_flag: Option<String>,
}

#[derive(Args, Debug, Default)]
pub(crate) struct VerifySlotArgs {
    #[command(flatten)]
    slot: SlotRefArgs,
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
pub(crate) struct SlotArgs {
    #[command(subcommand)]
    command: Option<SlotSubcommand>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum SlotSubcommand {
    #[command(about = "Plan or execute a managed workspace plus agents")]
    Spawn(SlotSpawnOptions),
    #[command(about = "Verify required local slot metadata")]
    Verify(VerifySlotArgs),
    #[command(about = "Plan or execute backend workspace/window close")]
    Close(SlotCloseArgs),
    #[command(about = "Print tab-separated slot rows")]
    List(FilterArgs),
    #[command(about = "Print all metadata for one slot")]
    Inspect(SlotRefArgs),
    #[command(about = "Print one slot's lifecycle class")]
    Classify(SlotRefArgs),
    #[command(about = "Mark a slot snoozed or another requested status")]
    Snooze(SlotMarkArgs),
    #[command(about = "Mark a slot blocked with optional reason")]
    Block(SlotMarkArgs),
    #[command(about = "Mark a slot active and clear snooze/block fields")]
    Resume(SlotMarkArgs),
    #[command(about = "Mark a slot ready for review")]
    Review(SlotMarkArgs),
    #[command(about = "Record a nudge or deliver a prompt to a slot role")]
    Nudge(SlotNudgeArgs),
    #[command(about = "Plan or execute git worktree removal")]
    RemoveWorktree(SlotWorktreeRemoveArgs),
    #[command(name = "worktree-remove", about = "Alias for remove-worktree")]
    WorktreeRemove(SlotWorktreeRemoveArgs),
    #[command(about = "Count slots matching class/status filters")]
    Count(FilterArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args, Debug, Default)]
pub(crate) struct FilterArgs {
    #[arg(long = "class", value_name = "CLASS", allow_hyphen_values = true)]
    class_filter: Option<String>,
    #[arg(long, value_name = "LIST", allow_hyphen_values = true)]
    status: Option<String>,
}

#[derive(Args, Debug, Default)]
pub(crate) struct JsonFilterArgs {
    #[command(flatten)]
    filter: FilterArgs,
    #[arg(long)]
    json: bool,
}

#[derive(Args, Debug)]
pub(crate) struct SlotMarkArgs {
    #[command(flatten)]
    slot: SlotRefArgs,
    #[arg(long, value_name = "LIST", allow_hyphen_values = true)]
    status: Option<String>,
    #[arg(long, value_name = "N", allow_hyphen_values = true)]
    days: Option<String>,
    #[arg(long, value_name = "EPOCH", allow_hyphen_values = true)]
    until: Option<String>,
    #[arg(long, value_name = "TEXT", allow_hyphen_values = true)]
    reason: Option<String>,
}

#[derive(Args, Debug)]
pub(crate) struct SlotNudgeArgs {
    #[command(flatten)]
    slot: SlotRefArgs,
    #[arg(long, value_name = "ROLE", allow_hyphen_values = true)]
    role: Option<String>,
    #[arg(long, value_name = "TEXT", allow_hyphen_values = true)]
    prompt: Option<String>,
    #[arg(long = "prompt-file", value_name = "PATH", allow_hyphen_values = true)]
    prompt_file: Option<String>,
}

#[derive(Args, Debug)]
pub(crate) struct SlotCloseArgs {
    #[command(flatten)]
    slot: SlotRefArgs,
    #[arg(long)]
    plan: bool,
    #[arg(long)]
    exec: bool,
}

#[derive(Args, Debug)]
pub(crate) struct SlotWorktreeRemoveArgs {
    #[command(flatten)]
    slot: SlotRefArgs,
    #[arg(long)]
    plan: bool,
    #[arg(long)]
    exec: bool,
    #[arg(long = "pull-main")]
    pull_main: bool,
}

#[derive(Args, Debug)]
pub(crate) struct CleanupArgs {
    #[command(subcommand)]
    command: Option<CleanupSubcommand>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum CleanupSubcommand {
    #[command(about = "List slots; defaults to --class done")]
    List(JsonFilterArgs),
    #[command(about = "Inspect one slot")]
    Inspect(SlotRefArgs),
    #[command(about = "Plan or execute backend close for one slot")]
    Close(SlotCloseArgs),
    #[command(about = "Print counts by lifecycle class")]
    Summary,
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args, Debug)]
pub(crate) struct RevampArgs {
    #[command(subcommand)]
    command: Option<RevampSubcommand>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum RevampSubcommand {
    #[command(about = "List stale slots by default")]
    List(FilterArgs),
    #[command(about = "Inspect one slot")]
    Inspect(SlotRefArgs),
    #[command(about = "Mark one slot snoozed or requested status")]
    Snooze(SlotMarkArgs),
    #[command(about = "Mark one slot blocked")]
    Block(SlotMarkArgs),
    #[command(about = "Mark one slot active")]
    Resume(SlotMarkArgs),
    #[command(about = "Record a nudge or deliver a prompt")]
    Nudge(SlotNudgeArgs),
    #[command(about = "Print counts by lifecycle class")]
    Summary,
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args, Debug)]
pub(crate) struct StatusArgs {
    #[command(subcommand)]
    command: Option<StatusSubcommand>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum StatusSubcommand {
    #[command(about = "Print tab-separated slot status rows")]
    List(JsonFilterArgs),
    #[command(about = "Print all metadata for one slot")]
    Inspect(SlotRefArgs),
    #[command(about = "Print locally tracked worktree rows")]
    Worktrees,
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args, Debug)]
pub(crate) struct CapArgs {
    #[command(subcommand)]
    command: Option<CapSubcommand>,
}

#[derive(Subcommand, Debug)]
pub(crate) enum CapSubcommand {
    #[command(about = "Print work/review counts and whether work capacity remains")]
    Check(CapCheckArgs),
    #[command(external_subcommand)]
    External(Vec<String>),
}

#[derive(Args, Debug, Default)]
pub(crate) struct CapCheckArgs {
    #[arg(long)]
    json: bool,
}

impl Commands {
    pub(crate) fn dispatch(self, home: &Path) -> Result<()> {
        match self {
            Self::Init => crate::commands::init::init(home),
            Self::Config(command) => crate::config::config_command(home, &mut command.into_args()),
            Self::Ledger(command) => {
                crate::commands::ledger::ledger_command(home, &mut command.into_args())
            }
            Self::State(command) => {
                crate::commands::state::state_command(home, &mut command.into_args())
            }
            Self::BoardCache(command) => {
                crate::commands::store::board_cache_command(home, &mut command.into_args())
            }
            Self::Capsule(command) => {
                crate::commands::store::capsule_command(home, &mut command.into_args())
            }
            Self::Prompt(command) => {
                crate::commands::store::prompt_command(&mut command.into_args())
            }
            Self::Backend(command) => {
                crate::commands::backend::backend_command(home, &mut command.into_args())
            }
            Self::Worktree(command) => {
                crate::commands::provision::worktree_command(home, &mut command.into_args())
            }
            Self::Workspace(command) => {
                crate::commands::slot::workspace_command(home, &mut command.into_args())
            }
            Self::Spawn(command) => {
                crate::commands::provision::spawn_command(home, &mut command.into_args())
            }
            Self::Agent(command) => {
                crate::commands::agent::agent_command(home, &mut command.into_args())
            }
            Self::Kickoff(command) => {
                crate::commands::agent::kickoff_command(home, &mut command.into_args())
            }
            Self::Verify(command) => {
                crate::commands::agent::verify_command(home, &mut command.into_args())
            }
            Self::Slot(command) => {
                crate::commands::slot::slot_command(home, &mut command.into_args())
            }
            Self::Cleanup(command) => {
                crate::commands::lifecycle::cleanup_command(home, &mut command.into_args())
            }
            Self::Revamp(command) => {
                crate::commands::lifecycle::revamp_command(home, &mut command.into_args())
            }
            Self::Status(command) => {
                crate::commands::lifecycle::status_command(home, &mut command.into_args())
            }
            Self::Cap(command) => {
                crate::commands::lifecycle::cap_command(home, &mut command.into_args())
            }
            Self::Statusline => crate::commands::diag::statusline(home),
            Self::WorkHours => crate::commands::diag::work_hours(home),
            Self::Doctor => crate::commands::diag::doctor(home),
            Self::Selftest => crate::commands::diag::selftest(home),
            Self::External(args) => Err(MaehError::Usage(format!(
                "unknown command {}",
                args.first().map_or("", String::as_str)
            ))),
        }
    }
}
impl ConfigArgs {
    fn into_args(self) -> Vec<String> {
        vec![match self.command {
            Some(ConfigSubcommand::Path) => "path".to_string(),
            Some(ConfigSubcommand::Show) => "show".to_string(),
            Some(ConfigSubcommand::Emit) => "emit".to_string(),
            Some(ConfigSubcommand::SetHome(command)) => {
                let mut args = vec!["set-home".to_string()];
                push_pos(&mut args, &command.path);
                return args;
            }
            Some(ConfigSubcommand::External(args)) => return args,
            None => return Vec::new(),
        }]
    }
}

impl LedgerArgs {
    fn into_args(self) -> Vec<String> {
        match self.command {
            Some(LedgerSubcommand::Append(command)) => {
                let mut args = vec!["append".to_string()];
                push_optional_maybe(&mut args, "--loop", &command.loop_name);
                push_optional(&mut args, "--event", &command.event);
                push_optional(&mut args, "--target", &command.target);
                push_optional(&mut args, "--data", &command.data);
                args
            }
            Some(LedgerSubcommand::List(command)) => {
                let mut args = vec!["list".to_string()];
                push_optional_maybe(&mut args, "--loop", &command.loop_name);
                args
            }
            Some(LedgerSubcommand::External(args)) => args,
            None => Vec::new(),
        }
    }
}

impl StateArgs {
    fn into_args(self) -> Vec<String> {
        match self.command {
            Some(StateSubcommand::Tag { slot, key, value }) => {
                vec!["tag".to_string(), slot, key, value]
            }
            Some(StateSubcommand::Untag { slot, key }) => vec!["untag".to_string(), slot, key],
            Some(StateSubcommand::Get { slot, key }) => vec!["get".to_string(), slot, key],
            Some(StateSubcommand::List) => vec!["list".to_string()],
            Some(StateSubcommand::Worktree { slot }) => vec!["worktree".to_string(), slot],
            Some(StateSubcommand::DeleteSlot { slot }) => vec!["delete-slot".to_string(), slot],
            Some(StateSubcommand::External(args)) => args,
            None => Vec::new(),
        }
    }
}

impl BoardCacheArgs {
    fn into_args(self) -> Vec<String> {
        match self.command {
            Some(BoardCacheSubcommand::Put(command)) => {
                let mut args = vec!["put".to_string()];
                push_optional(&mut args, "--key", &command.key);
                args
            }
            Some(BoardCacheSubcommand::Get(command)) => {
                let mut args = vec!["get".to_string()];
                push_optional(&mut args, "--key", &command.key);
                push_present(&mut args, "--stale", command.stale);
                args
            }
            Some(BoardCacheSubcommand::External(args)) => args,
            None => Vec::new(),
        }
    }
}

impl CapsuleArgs {
    fn into_args(self) -> Vec<String> {
        let (name, command) = match self.command {
            Some(CapsuleSubcommand::Put(command)) => ("put", command),
            Some(CapsuleSubcommand::Get(command)) => ("get", command),
            Some(CapsuleSubcommand::Prompt(command)) => ("prompt", command),
            Some(CapsuleSubcommand::External(args)) => return args,
            None => return Vec::new(),
        };
        let mut args = vec![name.to_string(), command.url];
        push_optional(&mut args, "--edited", &command.edited);
        args
    }
}

impl PromptArgs {
    fn into_args(self) -> Vec<String> {
        match self.command {
            Some(PromptSubcommand::Kickoff(command)) => {
                let mut args = vec!["kickoff".to_string()];
                push_optional(&mut args, "--url", &command.url);
                push_optional(&mut args, "--capsule-file", &command.capsule_file);
                args
            }
            Some(PromptSubcommand::External(args)) => args,
            None => Vec::new(),
        }
    }
}

impl BackendArgs {
    fn into_args(self) -> Vec<String> {
        let mut args = vec![match self.command {
            Some(BackendSubcommand::Plan) => "plan",
            Some(BackendSubcommand::Discover) => "discover",
            Some(BackendSubcommand::Reconcile) => "reconcile",
            Some(BackendSubcommand::ListTaskSlots) => "list-task-slots",
            Some(BackendSubcommand::ListWorktrees) => "list-worktrees",
            Some(BackendSubcommand::External(args)) => return args,
            None => return Vec::new(),
        }
        .to_string()];
        push_optional(&mut args, "--fixture", &self.fixture);
        push_present(&mut args, "--exec", self.exec);
        args
    }
}

impl WorktreeArgs {
    fn into_args(self) -> Vec<String> {
        match self.command {
            Some(WorktreeSubcommand::Plan(command)) => {
                let mut args = vec!["plan".to_string()];
                command.append(&mut args);
                args
            }
            Some(WorktreeSubcommand::Open(command)) => {
                let mut args = vec!["open".to_string()];
                command.append(&mut args);
                args
            }
            Some(WorktreeSubcommand::External(args)) => args,
            None => Vec::new(),
        }
    }
}

impl WorktreeOptions {
    fn append(&self, args: &mut Vec<String>) {
        push_optional(args, "--slot", &self.slot);
        push_optional(args, "--repo", &self.repo);
        push_optional(args, "--branch", &self.branch);
        push_optional(args, "--base", &self.base);
        push_optional(args, "--path", &self.path);
        push_optional(args, "--label", &self.label);
        push_present(args, "--create", self.create);
        self.layout.append(args);
    }
}

impl LayoutArgs {
    fn append(&self, args: &mut Vec<String>) {
        push_present(args, "--with-editor", self.with_editor);
        push_present(args, "--no-editor", self.no_editor);
        push_optional(args, "--editor", &self.editor);
        push_present(args, "--focus", self.focus);
        push_present(args, "--no-focus", self.no_focus);
    }
}

impl WorkspaceArgs {
    fn into_args(self) -> Vec<String> {
        match self.command {
            Some(WorkspaceSubcommand::Register(command)) => {
                let mut args = vec!["register".to_string()];
                push_pair(&mut args, "--slot", &command.slot);
                push_pair(&mut args, "--workspace", &command.workspace);
                push_pair(&mut args, "--worktree", &command.worktree);
                push_optional(&mut args, "--repo", &command.repo);
                push_optional(&mut args, "--task-url", &command.task_url);
                push_optional(&mut args, "--primary-pane", &command.primary_pane);
                push_optional(&mut args, "--critic-pane", &command.critic_pane);
                push_optional(&mut args, "--editor-pane", &command.editor_pane);
                push_optional(&mut args, "--backend", &command.backend);
                push_optional(&mut args, "--status", &command.status);
                args
            }
            Some(WorkspaceSubcommand::Spawn(command)) => {
                let mut args = vec!["spawn".to_string()];
                command.append(&mut args);
                args
            }
            Some(WorkspaceSubcommand::External(args)) => args,
            None => Vec::new(),
        }
    }
}

impl SpawnArgs {
    fn into_args(self) -> Vec<String> {
        match self.command {
            Some(SpawnSubcommand::Plan(command)) => {
                let mut args = vec!["plan".to_string()];
                command.append(&mut args);
                args
            }
            Some(SpawnSubcommand::Run(command)) => {
                let mut args = vec!["run".to_string()];
                command.append(&mut args);
                args
            }
            Some(SpawnSubcommand::External(args)) => args,
            None => Vec::new(),
        }
    }
}

impl SpawnOptions {
    fn append(&self, args: &mut Vec<String>) {
        push_pair(args, "--task-url", &self.task_url);
        self.worktree.append(args);
        push_optional(args, "--primary-cmd", &self.primary_cmd);
        push_optional(args, "--critic-cmd", &self.critic_cmd);
        push_optional(args, "--editor-cmd", &self.editor_cmd);
    }
}

impl SlotSpawnOptions {
    fn append(&self, args: &mut Vec<String>) {
        push_optional(args, "--backend", &self.backend);
        push_present(args, "--exec", self.exec);
        push_optional(args, "--slot", &self.slot);
        push_pair(args, "--repo", &self.repo);
        push_optional(args, "--branch", &self.branch);
        push_optional(args, "--base", &self.base);
        push_pair(args, "--path", &self.path);
        push_optional(args, "--label", &self.label);
        push_present(args, "--create", self.create);
        push_present(args, "--open-existing", self.open_existing);
        push_pair(args, "--task-url", &self.task_url);
        push_optional(args, "--primary-cmd", &self.primary_cmd);
        push_optional(args, "--critic-cmd", &self.critic_cmd);
        push_optional(args, "--editor-cmd", &self.editor_cmd);
        self.layout.append(args);
    }
}

impl AgentArgs {
    fn into_args(self) -> Vec<String> {
        match self.command {
            Some(AgentSubcommand::Deliver(command)) => {
                let mut args = vec!["deliver".to_string()];
                command.deliver.append(&mut args);
                push_present(&mut args, "--exec", command.exec);
                args
            }
            Some(AgentSubcommand::External(args)) => args,
            None => Vec::new(),
        }
    }
}

impl DeliverArgs {
    fn append(&self, args: &mut Vec<String>) {
        push_pos(args, &self.positional_target);
        push_pos(args, &self.positional_prompt);
        push_optional(args, "--target", &self.target);
        push_optional(args, "--slot", &self.slot);
        push_optional(args, "--role", &self.role);
        push_optional(args, "--prompt", &self.prompt);
        push_optional(args, "--prompt-file", &self.prompt_file);
        push_optional(args, "--pane-text", &self.pane_text);
        push_optional(args, "--pane-file", &self.pane_file);
    }
}

impl KickoffArgs {
    fn into_args(self) -> Vec<String> {
        match self.command {
            Some(KickoffSubcommand::Plan(command)) => {
                let mut args = vec!["plan".to_string()];
                command.append(&mut args);
                args
            }
            Some(KickoffSubcommand::Run(command)) => {
                let mut args = vec!["run".to_string()];
                command.append(&mut args);
                args
            }
            Some(KickoffSubcommand::External(args)) => args,
            None => Vec::new(),
        }
    }
}

impl VerifyArgs {
    fn into_args(self) -> Vec<String> {
        match self.command {
            Some(VerifySubcommand::Prompt(command)) => {
                let mut args = vec!["prompt".to_string()];
                push_optional(&mut args, "--before", &command.before);
                push_optional(&mut args, "--before-file", &command.before_file);
                push_optional(&mut args, "--after", &command.after);
                push_optional(&mut args, "--after-file", &command.after_file);
                push_optional(&mut args, "--prompt", &command.prompt);
                push_optional(&mut args, "--prompt-file", &command.prompt_file);
                args
            }
            Some(VerifySubcommand::Slot(command)) => {
                let mut args = vec!["slot".to_string()];
                command.append_value(&mut args);
                args
            }
            Some(VerifySubcommand::External(args)) => args,
            None => Vec::new(),
        }
    }
}

impl SlotArgs {
    fn into_args(self) -> Vec<String> {
        match self.command {
            Some(SlotSubcommand::Spawn(command)) => {
                let mut args = vec!["spawn".to_string()];
                command.append(&mut args);
                args
            }
            Some(SlotSubcommand::Verify(command)) => verify_slot_command_args("verify", command),
            Some(SlotSubcommand::Close(command)) => {
                let mut args = vec!["close".to_string()];
                command.append(&mut args);
                args
            }
            Some(SlotSubcommand::List(filter)) => filter_args("list", filter),
            Some(SlotSubcommand::Inspect(slot)) => slot_command_args("inspect", slot),
            Some(SlotSubcommand::Classify(slot)) => slot_command_args("classify", slot),
            Some(SlotSubcommand::Snooze(command)) => mark_args("snooze", command),
            Some(SlotSubcommand::Block(command)) => mark_args("block", command),
            Some(SlotSubcommand::Resume(command)) => mark_args("resume", command),
            Some(SlotSubcommand::Review(command)) => mark_args("review", command),
            Some(SlotSubcommand::Nudge(command)) => {
                let mut args = vec!["nudge".to_string()];
                command.append(&mut args);
                args
            }
            Some(SlotSubcommand::RemoveWorktree(command)) => {
                let mut args = vec!["remove-worktree".to_string()];
                command.append(&mut args);
                args
            }
            Some(SlotSubcommand::WorktreeRemove(command)) => {
                let mut args = vec!["worktree-remove".to_string()];
                command.append(&mut args);
                args
            }
            Some(SlotSubcommand::Count(filter)) => filter_args("count", filter),
            Some(SlotSubcommand::External(args)) => args,
            None => Vec::new(),
        }
    }
}

impl SlotRefArgs {
    fn append_value(&self, args: &mut Vec<String>) {
        if let Some(slot) = &self.slot_flag {
            args.push(slot.clone());
        } else {
            push_pos(args, &self.slot);
        }
    }

    fn append_flag_or_value(&self, args: &mut Vec<String>) {
        if let Some(slot) = &self.slot_flag {
            push_pair(args, "--slot", slot);
        } else {
            push_pos(args, &self.slot);
        }
    }
}

impl VerifySlotArgs {
    fn append_value(&self, args: &mut Vec<String>) {
        self.slot.append_value(args);
        push_present(args, "--json", self.json);
    }

    fn append_flag_or_value(&self, args: &mut Vec<String>) {
        self.slot.append_flag_or_value(args);
        push_present(args, "--json", self.json);
    }
}

impl SlotMarkArgs {
    fn append(&self, args: &mut Vec<String>) {
        self.slot.append_flag_or_value(args);
        push_optional(args, "--status", &self.status);
        push_optional(args, "--days", &self.days);
        push_optional(args, "--until", &self.until);
        push_optional(args, "--reason", &self.reason);
    }
}

impl SlotNudgeArgs {
    fn append(&self, args: &mut Vec<String>) {
        self.slot.append_flag_or_value(args);
        push_optional(args, "--role", &self.role);
        push_optional(args, "--prompt", &self.prompt);
        push_optional(args, "--prompt-file", &self.prompt_file);
    }
}

impl SlotCloseArgs {
    fn append(&self, args: &mut Vec<String>) {
        self.slot.append_flag_or_value(args);
        push_present(args, "--plan", self.plan);
        push_present(args, "--exec", self.exec);
    }
}

impl SlotWorktreeRemoveArgs {
    fn append(&self, args: &mut Vec<String>) {
        self.slot.append_flag_or_value(args);
        push_present(args, "--plan", self.plan);
        push_present(args, "--exec", self.exec);
        push_present(args, "--pull-main", self.pull_main);
    }
}

impl CleanupArgs {
    fn into_args(self) -> Vec<String> {
        match self.command {
            Some(CleanupSubcommand::List(filter)) => json_filter_args("list", filter),
            Some(CleanupSubcommand::Inspect(slot)) => slot_command_args("inspect", slot),
            Some(CleanupSubcommand::Close(command)) => {
                let mut args = vec!["close".to_string()];
                command.append(&mut args);
                args
            }
            Some(CleanupSubcommand::Summary) => vec!["summary".to_string()],
            Some(CleanupSubcommand::External(args)) => args,
            None => Vec::new(),
        }
    }
}

impl RevampArgs {
    fn into_args(self) -> Vec<String> {
        match self.command {
            Some(RevampSubcommand::List(filter)) => filter_args("list", filter),
            Some(RevampSubcommand::Inspect(slot)) => slot_command_args("inspect", slot),
            Some(RevampSubcommand::Snooze(command)) => mark_args("snooze", command),
            Some(RevampSubcommand::Block(command)) => mark_args("block", command),
            Some(RevampSubcommand::Resume(command)) => mark_args("resume", command),
            Some(RevampSubcommand::Nudge(command)) => {
                let mut args = vec!["nudge".to_string()];
                command.append(&mut args);
                args
            }
            Some(RevampSubcommand::Summary) => vec!["summary".to_string()],
            Some(RevampSubcommand::External(args)) => args,
            None => Vec::new(),
        }
    }
}

impl StatusArgs {
    fn into_args(self) -> Vec<String> {
        match self.command {
            Some(StatusSubcommand::List(filter)) => json_filter_args("list", filter),
            Some(StatusSubcommand::Inspect(slot)) => slot_command_args("inspect", slot),
            Some(StatusSubcommand::Worktrees) => vec!["worktrees".to_string()],
            Some(StatusSubcommand::External(args)) => args,
            None => Vec::new(),
        }
    }
}

impl CapArgs {
    fn into_args(self) -> Vec<String> {
        match self.command {
            Some(CapSubcommand::Check(command)) => {
                let mut args = vec!["check".to_string()];
                push_present(&mut args, "--json", command.json);
                args
            }
            Some(CapSubcommand::External(args)) => args,
            None => Vec::new(),
        }
    }
}

fn slot_command_args(command: &str, slot: SlotRefArgs) -> Vec<String> {
    let mut args = vec![command.to_string()];
    slot.append_flag_or_value(&mut args);
    args
}

fn verify_slot_command_args(command: &str, slot: VerifySlotArgs) -> Vec<String> {
    let mut args = vec![command.to_string()];
    slot.append_flag_or_value(&mut args);
    args
}

fn filter_args(command: &str, filter: FilterArgs) -> Vec<String> {
    let mut args = vec![command.to_string()];
    push_optional(&mut args, "--class", &filter.class_filter);
    push_optional(&mut args, "--status", &filter.status);
    args
}

fn json_filter_args(command: &str, filter: JsonFilterArgs) -> Vec<String> {
    let mut args = filter_args(command, filter.filter);
    push_present(&mut args, "--json", filter.json);
    args
}

fn mark_args(command: &str, mark: SlotMarkArgs) -> Vec<String> {
    let mut args = vec![command.to_string()];
    mark.append(&mut args);
    args
}

fn push_pair(args: &mut Vec<String>, flag: &str, value: &str) {
    args.push(flag.to_string());
    args.push(value.to_string());
}

fn push_optional(args: &mut Vec<String>, flag: &str, value: &Option<String>) {
    if let Some(value) = value {
        push_pair(args, flag, value);
    }
}

fn push_optional_maybe(args: &mut Vec<String>, flag: &str, value: &Option<Option<String>>) {
    match value {
        Some(Some(value)) => push_pair(args, flag, value),
        Some(None) => args.push(flag.to_string()),
        None => {}
    }
}

fn push_present(args: &mut Vec<String>, flag: &str, present: bool) {
    if present {
        args.push(flag.to_string());
    }
}

fn push_pos(args: &mut Vec<String>, value: &Option<String>) {
    if let Some(value) = value {
        args.push(value.clone());
    }
}
