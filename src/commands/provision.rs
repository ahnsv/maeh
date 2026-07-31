use std::path::{Path, PathBuf};

use maeh::backend::{
    adapter_for, print_operations, BackendError, CommandOutput, CommandRunner, CommandSpec,
    LayoutOptions, RealRunner, SpawnRecord, SpawnRequest, WorktreeRecord, WorktreeRequest,
};

use crate::commands::state::{read_state, write_state};
use crate::config::{
    backend_settings_for_config, parse_bool, print_backend_resolution, read_config, Config,
};
use crate::error::{MaehError, Result};
use crate::util::{flag_present, flag_value, take_arg};

pub(crate) fn worktree_command(home: &Path, args: &mut Vec<String>) -> Result<()> {
    let command = take_arg(args, "worktree command")?;
    if !matches!(command.as_str(), "plan" | "open") {
        return Err(MaehError::Usage(format!(
            "unknown worktree command {command}"
        )));
    }
    let config = read_config(home)?;
    let request = worktree_request(&config, args)?;
    let settings = backend_settings_for_config(&config)?;
    let adapter = adapter_for(&settings);
    println!("maeh worktree {command}");
    print_backend_resolution(&settings);
    if command == "plan" {
        print_operations(&adapter.worktree_plan(&request));
    } else {
        let mut runner = RealRunner;
        let record = adapter.execute_worktree(&mut runner, &request)?;
        persist_worktree(home, &record, "", &request.repo)?;
        print_worktree_record(&record);
    }
    Ok(())
}

pub(crate) fn spawn_command(home: &Path, args: &mut Vec<String>) -> Result<()> {
    let command = take_arg(args, "spawn command")?;
    if !matches!(command.as_str(), "plan" | "run") {
        return Err(MaehError::Usage(format!("unknown spawn command {command}")));
    }
    let config = read_config(home)?;
    let request = spawn_request(&config, args)?;
    let settings = backend_settings_for_config(&config)?;
    let adapter = adapter_for(&settings);
    println!("maeh spawn {command}");
    print_backend_resolution(&settings);
    if command == "plan" {
        print_operations(&adapter.spawn_plan(&request));
    } else {
        let mut runner = RealRunner;
        let record = adapter.execute_spawn(&mut runner, &request)?;
        persist_spawn(home, &record, &request.task_url, &request.worktree.repo)?;
        print_spawn_record(&record);
    }
    Ok(())
}

pub(crate) fn worktree_request(config: &Config, args: &mut Vec<String>) -> Result<WorktreeRequest> {
    let slot = required_flag(args, "--slot")?;
    let repo = PathBuf::from(required_flag(args, "--repo")?);
    let branch = flag_value(args, "--branch", "")?;
    let base = flag_value(args, "--base", "HEAD")?;
    let path = PathBuf::from(required_flag(args, "--path")?);
    let label = flag_value(args, "--label", &slot)?;
    let create = flag_present(args, "--create");
    let layout = layout_options(config, args);
    Ok(WorktreeRequest {
        slot,
        repo,
        branch,
        base,
        path,
        label,
        create,
        layout,
    })
}

pub(crate) fn spawn_request(config: &Config, args: &mut Vec<String>) -> Result<SpawnRequest> {
    let task_url = required_flag(args, "--task-url")?;
    let worktree = worktree_request(config, args)?;
    let primary_arg = flag_value(args, "--primary-cmd", &config.agents.primary_cmd)?;
    let primary_agent_cmd = command_words(&primary_arg);
    let critic_arg = flag_value(args, "--critic-cmd", &config.agents.critic_cmd)?;
    let critic_agent_cmd = command_words(&critic_arg);
    let editor_arg = flag_value(args, "--editor-cmd", &config.agents.editor_cmd)?;
    let editor_cmd = command_words(&editor_arg);
    Ok(SpawnRequest {
        worktree,
        task_url,
        primary_agent_cmd,
        critic_agent_cmd,
        editor_cmd,
    })
}

pub(crate) fn layout_options(config: &Config, args: &mut Vec<String>) -> LayoutOptions {
    let mut include_editor = config.layout.include_editor;
    if flag_present(args, "--no-editor") {
        include_editor = false;
    }
    if flag_present(args, "--with-editor") {
        include_editor = true;
    }
    if args.iter().any(|arg| arg == "--editor") {
        include_editor = parse_bool(
            &flag_value(args, "--editor", "true").unwrap_or_default(),
            include_editor,
        );
    }
    let mut focus = config.layout.focus;
    if flag_present(args, "--focus") {
        focus = true;
    }
    if flag_present(args, "--no-focus") {
        focus = false;
    }
    LayoutOptions {
        include_editor,
        focus,
    }
}

pub(crate) fn command_words(command: &str) -> Vec<String> {
    command
        .split_whitespace()
        .map(ToString::to_string)
        .collect()
}

pub(crate) fn required_flag(args: &mut Vec<String>, flag: &str) -> Result<String> {
    let value = flag_value(args, flag, "")?;
    if value.is_empty() {
        Err(MaehError::Usage(format!("{flag} needs a value")))
    } else {
        Ok(value)
    }
}

pub(crate) fn run_command(
    runner: &mut dyn CommandRunner,
    spec: &CommandSpec,
) -> Result<CommandOutput> {
    let output = runner.run(spec)?;
    if output.status != 0 {
        return Err(BackendError::CommandFailed {
            program: spec.program.clone(),
            status: output.status,
        }
        .into());
    }
    Ok(output)
}

pub(crate) fn persist_worktree(
    home: &Path,
    record: &WorktreeRecord,
    task_url: &str,
    repo: &Path,
) -> Result<()> {
    let mut state = read_state(home)?;
    let entry = state.entry(record.slot.clone()).or_default();
    entry.insert("backend".to_string(), record.backend.to_string());
    entry.insert("workspace_id".to_string(), record.workspace_id.clone());
    entry.insert("worktree".to_string(), record.worktree.clone());
    entry.insert("repo".to_string(), repo.display().to_string());
    if !record.window_id.is_empty() {
        entry.insert("window_id".to_string(), record.window_id.clone());
    }
    if !task_url.is_empty() {
        entry.insert("task_url".to_string(), task_url.to_string());
    }
    write_state(home, &state)
}

pub(crate) fn persist_spawn(
    home: &Path,
    record: &SpawnRecord,
    task_url: &str,
    repo: &Path,
) -> Result<()> {
    persist_worktree(home, &record.worktree, task_url, repo)?;
    let mut state = read_state(home)?;
    let entry = state.entry(record.worktree.slot.clone()).or_default();
    entry.insert("primary_pane".to_string(), record.primary_pane.clone());
    entry.insert("critic_pane".to_string(), record.critic_pane.clone());
    if !record.editor_pane.is_empty() {
        entry.insert("editor_pane".to_string(), record.editor_pane.clone());
    }
    entry.insert("status".to_string(), "active".to_string());
    write_state(home, &state)
}

pub(crate) fn print_worktree_record(record: &WorktreeRecord) {
    println!("worktree opened");
    println!("  slot: {}", record.slot);
    println!("  workspace: {}", record.workspace_id);
    if !record.window_id.is_empty() {
        println!("  window: {}", record.window_id);
    }
    println!("  path: {}", record.worktree);
}

pub(crate) fn print_spawn_record(record: &SpawnRecord) {
    print_worktree_record(&record.worktree);
    println!("  primary pane: {}", record.primary_pane);
    println!("  critic pane: {}", record.critic_pane);
    if !record.editor_pane.is_empty() {
        println!("  editor pane: {}", record.editor_pane);
    }
}

pub(crate) fn persist_request_metadata(home: &Path, request: &SpawnRequest) -> Result<()> {
    let mut state = read_state(home)?;
    let entry = state.entry(request.worktree.slot.clone()).or_default();
    entry.insert("label".to_string(), request.worktree.label.clone());
    entry.insert("branch".to_string(), request.worktree.branch.clone());
    write_state(home, &state)
}
