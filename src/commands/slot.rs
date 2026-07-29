use std::collections::BTreeMap;
use std::path::Path;

use maeh::backend::{
    adapter_for, print_operations, BackendEnv, BackendKind, BackendSettings, BackendSlot,
    CommandSpec, OperationPlan, RealRunner,
};
use serde::Serialize;

use crate::commands::agent::{deliver_command, verify_slot};
use crate::commands::provision::{
    persist_request_metadata, persist_spawn, print_spawn_record, run_command, spawn_request,
};
use crate::commands::state::{read_state, state_delete_slot, write_state};
use crate::config::{
    backend_settings_for_config, backend_settings_for_config_env, read_config, Config,
};
use crate::error::{MaehError, Result};
use crate::util::{flag_present, flag_value, now_epoch, take_arg};

use crate::commands::ledger::append_ledger;

pub(crate) fn workspace_command(home: &Path, args: &mut Vec<String>) -> Result<()> {
    match take_arg(args, "workspace command")?.as_str() {
        "spawn" => workspace_spawn(home, args),
        "register" => workspace_register(home, args),
        other => Err(MaehError::Usage(format!(
            "unknown workspace command {other}"
        ))),
    }
}

pub(crate) fn workspace_spawn(home: &Path, args: &mut Vec<String>) -> Result<()> {
    slot_spawn_with_label(home, "workspace spawn", args)
}

pub(crate) fn workspace_register(home: &Path, args: &mut Vec<String>) -> Result<()> {
    let slot = crate::commands::provision::required_flag(args, "--slot")?;
    let config = read_config(home)?;
    let backend = flag_value(args, "--backend", &config.backend.to_string())?;
    let workspace = crate::commands::provision::required_flag(args, "--workspace")?;
    let worktree = crate::commands::provision::required_flag(args, "--worktree")?;
    let task_url = flag_value(args, "--task-url", "")?;
    let primary = flag_value(args, "--primary-pane", "")?;
    let critic = flag_value(args, "--critic-pane", "")?;
    let editor = flag_value(args, "--editor-pane", "")?;
    let repo = flag_value(args, "--repo", "")?;
    let status = flag_value(args, "--status", "active")?;
    let mut state = read_state(home)?;
    let entry = state.entry(slot.clone()).or_default();
    entry.insert("backend".to_string(), backend);
    entry.insert("workspace_id".to_string(), workspace.clone());
    entry.insert("worktree".to_string(), worktree);
    entry.insert("status".to_string(), status);
    if !task_url.is_empty() {
        entry.insert("task_url".to_string(), task_url);
    }
    if !primary.is_empty() {
        entry.insert("primary_pane".to_string(), primary);
    }
    if !critic.is_empty() {
        entry.insert("critic_pane".to_string(), critic);
    }
    if !editor.is_empty() {
        entry.insert("editor_pane".to_string(), editor);
    }
    if !repo.is_empty() {
        entry.insert("repo".to_string(), repo);
    }
    write_state(home, &state)?;
    println!("workspace registered");
    println!("  slot: {slot}");
    println!("  workspace: {workspace}");
    Ok(())
}

pub(crate) fn slot_command(home: &Path, args: &mut Vec<String>) -> Result<()> {
    match take_arg(args, "slot command")?.as_str() {
        "spawn" => slot_spawn_with_label(home, "slot spawn", args),
        "verify" => {
            let json = flag_present(args, "--json");
            let slot = slot_arg(args)?;
            verify_slot(home, &slot, json)
        }
        "close" => slot_close(home, args),
        "list" => print_slot_rows(
            home,
            &flag_value(args, "--class", "all")?,
            &flag_value(args, "--status", "")?,
            0,
            false,
        ),
        "inspect" => {
            let slot = slot_arg(args)?;
            slot_inspect(home, &slot)
        }
        "classify" => {
            let slot = slot_arg(args)?;
            slot_classify(home, &slot)
        }
        "snooze" => {
            let slot = slot_arg(args)?;
            slot_mark(home, &slot, "snoozed", args)
        }
        "block" => {
            let slot = slot_arg(args)?;
            slot_mark(home, &slot, "blocked", args)
        }
        "resume" => {
            let slot = slot_arg(args)?;
            slot_mark(home, &slot, "active", args)
        }
        "review" => {
            let slot = slot_arg(args)?;
            slot_mark(home, &slot, "review", args)
        }
        "nudge" => slot_nudge(home, args),
        "remove-worktree" | "worktree-remove" => slot_worktree_remove(home, args),
        "count" => slot_count(
            home,
            &flag_value(args, "--class", "all")?,
            &flag_value(args, "--status", "")?,
        ),
        other => Err(MaehError::Usage(format!("unknown slot command {other}"))),
    }
}

pub(crate) fn slot_spawn_with_label(
    home: &Path,
    label: &str,
    args: &mut Vec<String>,
) -> Result<()> {
    let exec = flag_present(args, "--exec");
    let backend = flag_value(args, "--backend", "")?;
    let mut config = read_config(home)?;
    match backend.as_str() {
        "" => {}
        value => config.backend = value.parse()?,
    }
    if !args.iter().any(|arg| arg == "--slot") {
        let default_slot = flag_value(args, "--label", "")?;
        if !default_slot.is_empty() {
            args.extend([
                "--slot".to_string(),
                default_slot.clone(),
                "--label".to_string(),
                default_slot,
            ]);
        }
    }
    if !args.iter().any(|arg| arg == "--create") && !flag_present(args, "--open-existing") {
        args.push("--create".to_string());
    }
    let request = spawn_request(&config, args)?;
    let settings = backend_settings_for_config(&config)?;
    let adapter = adapter_for(&settings);
    println!("maeh {label}");
    crate::config::print_backend_resolution(&settings);
    if exec {
        let mut runner = RealRunner;
        let record = adapter.execute_spawn(&mut runner, &request)?;
        persist_spawn(home, &record, &request.task_url, &request.worktree.repo)?;
        persist_request_metadata(home, &request)?;
        print_spawn_record(&record);
    } else {
        print_operations(&adapter.spawn_plan(&request));
    }
    Ok(())
}

pub(crate) fn slot_inspect(home: &Path, slot: &str) -> Result<()> {
    let state = read_state(home)?;
    let entry = state
        .get(slot)
        .ok_or_else(|| MaehError::CacheMiss(slot.to_string()))?;
    println!("slot: {slot}");
    for (key, value) in entry {
        println!("  {key}: {value}");
    }
    println!("  class: {}", slot_class(entry, now_epoch()));
    Ok(())
}

pub(crate) fn slot_classify(home: &Path, slot: &str) -> Result<()> {
    let state = read_state(home)?;
    let entry = state
        .get(slot)
        .ok_or_else(|| MaehError::CacheMiss(slot.to_string()))?;
    println!("{}\t{}", slot, slot_class(entry, now_epoch()));
    Ok(())
}

pub(crate) fn slot_mark(
    home: &Path,
    slot: &str,
    status: &str,
    args: &mut Vec<String>,
) -> Result<()> {
    let requested_status = flag_value(args, "--status", status)?;
    let until = if args.iter().any(|arg| arg == "--days") {
        let days = flag_value(args, "--days", "0")?.parse::<u64>().unwrap_or(0);
        now_epoch()
            .saturating_add(days.saturating_mul(86_400))
            .to_string()
    } else {
        flag_value(args, "--until", "0")?
    };
    let mut state = read_state(home)?;
    let entry = state.entry(slot.to_string()).or_default();
    entry.insert("status".to_string(), requested_status.clone());
    if status == "snoozed" || until != "0" {
        entry.insert("snooze_until".to_string(), until);
    }
    if requested_status == "blocked" {
        entry.insert(
            "block_reason".to_string(),
            flag_value(args, "--reason", "")?,
        );
    }
    if requested_status == "active" {
        entry.remove("snooze_until");
        entry.remove("block_reason");
    }
    write_state(home, &state)?;
    append_ledger(home, "slot", &requested_status, slot, "{}")?;
    Ok(())
}

pub(crate) fn slot_nudge(home: &Path, args: &mut Vec<String>) -> Result<()> {
    let slot = slot_arg(args)?;
    let prompt = crate::commands::agent::prompt_text(args)?;
    let role = flag_value(args, "--role", "primary")?;
    if prompt.is_empty() {
        let mut state = read_state(home)?;
        state
            .entry(slot.clone())
            .or_default()
            .insert("nudge_epoch".to_string(), now_epoch().to_string());
        write_state(home, &state)?;
        append_ledger(home, "slot", "nudge", &slot, "{}")?;
        return Ok(());
    }
    let mut deliver_args = vec![
        "--slot".to_string(),
        slot,
        "--role".to_string(),
        role,
        "--prompt".to_string(),
        prompt,
    ];
    deliver_command(home, "slot nudge", &mut deliver_args, true)
}

pub(crate) fn slot_close(home: &Path, args: &mut Vec<String>) -> Result<()> {
    let slot = slot_arg(args)?;
    let plan = flag_present(args, "--plan");
    let exec = flag_present(args, "--exec") && !plan;
    let config = read_config(home)?;
    let state = read_state(home)?;
    let entry = state
        .get(&slot)
        .ok_or_else(|| MaehError::CacheMiss(slot.clone()))?;
    let settings = settings_for_entry(&config, entry)?;
    let adapter = adapter_for(&settings);
    let target = entry
        .get("workspace_id")
        .or_else(|| entry.get("window_id"))
        .ok_or_else(|| MaehError::CacheMiss(format!("{slot}:workspace_id")))?;
    let spec = adapter.close_spec(target);
    let operation = OperationPlan::mutate_command(
        "close-slot",
        &slot,
        format!("close backend workspace/window {target}"),
        spec.clone(),
    );
    print_operations(std::slice::from_ref(&operation));
    if !exec {
        return Ok(());
    }
    let mut runner = RealRunner;
    let _ = run_command(&mut runner, &spec)?;
    state_delete_slot(home, &slot)?;
    Ok(())
}

pub(crate) fn slot_worktree_remove(home: &Path, args: &mut Vec<String>) -> Result<()> {
    let slot = slot_arg(args)?;
    let plan = flag_present(args, "--plan");
    let pull_main = flag_present(args, "--pull-main");
    let exec = flag_present(args, "--exec") && !plan;
    let state = read_state(home)?;
    let entry = state
        .get(&slot)
        .ok_or_else(|| MaehError::CacheMiss(slot.clone()))?;
    let repo = entry
        .get("repo")
        .ok_or_else(|| MaehError::CacheMiss(format!("{slot}:repo")))?;
    let worktree = entry
        .get("worktree")
        .ok_or_else(|| MaehError::CacheMiss(format!("{slot}:worktree")))?;
    let mut operations = Vec::new();
    if pull_main {
        operations.push(OperationPlan::mutate_command(
            "pull-main",
            &slot,
            format!("git -C {repo} pull --ff-only origin main"),
            CommandSpec::from_args(
                "git",
                vec![
                    "-C".to_string(),
                    repo.clone(),
                    "pull".to_string(),
                    "--ff-only".to_string(),
                    "origin".to_string(),
                    "main".to_string(),
                ],
            ),
        ));
    }
    operations.push(OperationPlan::mutate_command(
        "remove-worktree",
        &slot,
        format!("git worktree remove {worktree}"),
        CommandSpec::from_args(
            "git",
            vec![
                "-C".to_string(),
                repo.clone(),
                "worktree".to_string(),
                "remove".to_string(),
                worktree.clone(),
            ],
        ),
    ));
    print_operations(&operations);
    if exec {
        let mut runner = RealRunner;
        for spec in operations
            .into_iter()
            .filter_map(|operation| operation.command)
        {
            let _ = run_command(&mut runner, &spec)?;
        }
    }
    Ok(())
}

#[derive(Serialize)]
pub(crate) struct SlotRow {
    slot: String,
    task_url: String,
    status: String,
    snooze_until: String,
    age_secs: u64,
    class: String,
    label: String,
    worktree: String,
    primary_pane: String,
    critic_pane: String,
    repo: String,
}

pub(crate) fn print_slot_rows(
    home: &Path,
    class_filter: &str,
    status_filter: &str,
    stale_secs: u64,
    json: bool,
) -> Result<()> {
    let rows = slot_rows(home, class_filter, status_filter, stale_secs)?;
    if json {
        println!("{}", serde_json::to_string(&rows)?);
        return Ok(());
    }
    for row in rows {
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            row.slot,
            row.task_url,
            row.status,
            row.snooze_until,
            row.age_secs,
            row.class,
            row.label,
            row.worktree,
            row.primary_pane,
            row.critic_pane,
            row.repo,
        );
    }
    Ok(())
}

fn slot_rows(
    home: &Path,
    class_filter: &str,
    status_filter: &str,
    stale_secs: u64,
) -> Result<Vec<SlotRow>> {
    let now = now_epoch();
    let mut rows = Vec::new();
    for (slot, entry) in read_state(home)? {
        let class = slot_class_with_stale(&entry, now, stale_secs);
        if class_filter != "all" && class != class_filter {
            continue;
        }
        if !status_matches(&entry, status_filter) {
            continue;
        }
        rows.push(SlotRow {
            slot,
            task_url: entry_value(&entry, "task_url", ""),
            status: entry_value(&entry, "status", "none"),
            snooze_until: entry_value(&entry, "snooze_until", "0"),
            age_secs: slot_age(&entry, now),
            class,
            label: entry_value(&entry, "label", ""),
            worktree: entry_value(&entry, "worktree", ""),
            primary_pane: entry_value(&entry, "primary_pane", ""),
            critic_pane: entry_value(&entry, "critic_pane", ""),
            repo: entry_value(&entry, "repo", ""),
        });
    }
    Ok(rows)
}

pub(crate) fn print_task_slot_rows(home: &Path) -> Result<()> {
    for (slot, entry) in read_state(home)? {
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            slot,
            entry.get("task_url").map_or("", String::as_str),
            entry.get("status").map_or("none", String::as_str),
            entry.get("snooze_until").map_or("0", String::as_str),
            slot_age(&entry, now_epoch()),
            entry.get("label").map_or("", String::as_str),
            entry.get("primary_pane").map_or("", String::as_str),
            entry.get("critic_pane").map_or("", String::as_str),
            entry.get("worktree").map_or("", String::as_str),
        );
    }
    Ok(())
}

pub(crate) fn print_backend_task_slots(slots: &[BackendSlot]) {
    for slot in slots {
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            slot.slot,
            slot.task_url,
            slot.status,
            slot.snooze_until,
            slot.age_secs,
            slot.name,
            slot.primary_pane,
            slot.critic_pane,
            slot.worktree
        );
    }
}

pub(crate) fn print_worktree_rows(home: &Path) -> Result<()> {
    for (slot, entry) in read_state(home)? {
        if !entry.contains_key("worktree") {
            continue;
        }
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            slot,
            entry.get("label").map_or("", String::as_str),
            entry.get("repo").map_or("", String::as_str),
            entry.get("branch").map_or("", String::as_str),
            entry.get("dirty").map_or("unknown", String::as_str),
            entry.get("worktree").map_or("", String::as_str),
        );
    }
    Ok(())
}

pub(crate) fn slot_count(home: &Path, class_filter: &str, status_filter: &str) -> Result<()> {
    let mut count = 0;
    for entry in read_state(home)?.into_values() {
        let class = slot_class(&entry, now_epoch());
        if (class_filter == "all" || class_filter == class) && status_matches(&entry, status_filter)
        {
            count += 1;
        }
    }
    println!("{count}");
    Ok(())
}

pub(crate) fn slot_arg(args: &mut Vec<String>) -> Result<String> {
    let value = flag_value(args, "--slot", "")?;
    if value.is_empty() {
        take_arg(args, "slot")
    } else {
        Ok(value)
    }
}

fn status_matches(entry: &BTreeMap<String, String>, status_filter: &str) -> bool {
    if status_filter.is_empty() {
        return true;
    }
    let status = entry.get("status").map_or("none", String::as_str);
    status_filter
        .split(',')
        .any(|candidate| candidate == status)
}

pub(crate) fn slot_class(entry: &BTreeMap<String, String>, now: u64) -> String {
    slot_class_with_stale(entry, now, 86_400)
}

fn slot_class_with_stale(entry: &BTreeMap<String, String>, now: u64, stale_secs: u64) -> String {
    match entry.get("status").map(String::as_str) {
        Some("done") => "done".to_string(),
        Some("blocked") => "blocked".to_string(),
        Some("snoozed") => "snoozed".to_string(),
        _ if stale_secs > 0 && slot_age(entry, now) > stale_secs => "stale".to_string(),
        Some(status) => status.to_string(),
        None => "none".to_string(),
    }
}

fn slot_age(entry: &BTreeMap<String, String>, now: u64) -> u64 {
    entry
        .get("last_activity_epoch")
        .and_then(|value| value.parse::<u64>().ok())
        .map_or(0, |last| now.saturating_sub(last))
}

fn entry_value(entry: &BTreeMap<String, String>, key: &str, default: &str) -> String {
    entry
        .get(key)
        .cloned()
        .unwrap_or_else(|| default.to_string())
}

fn settings_for_entry(
    config: &Config,
    entry: &BTreeMap<String, String>,
) -> Result<BackendSettings> {
    let env = BackendEnv::from_env()?;
    let mut settings = backend_settings_for_config_env(config, &env)?;
    if let Some(backend) = entry.get("backend") {
        let kind = backend.parse::<BackendKind>()?;
        settings.requested = kind;
        settings.selected = kind.resolve(&env);
    }
    Ok(settings)
}
