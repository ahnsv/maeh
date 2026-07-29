use std::fs;
use std::path::Path;

use maeh::backend::{
    adapter_for, delivery_plan, pane_text_from_read_output, print_operations,
    verify_prompt_execution, RealRunner,
};
use serde::Serialize;

use crate::commands::provision::{required_flag, run_command};
use crate::commands::state::read_state;
use crate::config::{backend_settings, print_backend_resolution};
use crate::error::{MaehError, Result};
use crate::util::{flag_present, flag_value, take_arg};

pub(crate) fn agent_command(home: &Path, args: &mut Vec<String>) -> Result<()> {
    match take_arg(args, "agent command")?.as_str() {
        "deliver" => {
            let exec = crate::util::flag_present(args, "--exec");
            deliver_command(home, "agent deliver", args, exec)
        }
        other => Err(MaehError::Usage(format!("unknown agent command {other}"))),
    }
}

pub(crate) fn kickoff_command(home: &Path, args: &mut Vec<String>) -> Result<()> {
    let command = take_arg(args, "kickoff command")?;
    match command.as_str() {
        "plan" => deliver_command(home, "kickoff plan", args, false),
        "run" => deliver_command(home, "kickoff run", args, true),
        other => Err(MaehError::Usage(format!("unknown kickoff command {other}"))),
    }
}

pub(crate) fn deliver_command(
    home: &Path,
    label: &str,
    args: &mut Vec<String>,
    exec: bool,
) -> Result<()> {
    let positional_target = if args.first().is_some_and(|arg| !arg.starts_with('-')) {
        Some(take_arg(args, "target")?)
    } else {
        None
    };
    let positional_prompt = if args.first().is_some_and(|arg| !arg.starts_with('-')) {
        Some(take_arg(args, "prompt")?)
    } else {
        None
    };
    let target = positional_target.unwrap_or(flag_value(args, "--target", "")?);
    let slot = flag_value(args, "--slot", "")?;
    if target.is_empty() && slot.is_empty() {
        return Err(MaehError::Usage(
            "--target or --slot needs a value".to_string(),
        ));
    }
    let role = flag_value(args, "--role", "both")?;
    let prompt = positional_prompt.unwrap_or(prompt_text(args)?);
    let pane_text = pane_text(args)?;
    let settings = backend_settings(home)?;
    let adapter = adapter_for(&settings);
    let targets = delivery_targets(home, &target, &slot, &role)?;
    println!("maeh {label}");
    print_backend_resolution(&settings);
    let mut runner = RealRunner;
    for deliver_target in targets {
        let live_text = if exec && pane_text.is_empty() {
            let output = run_command(&mut runner, &adapter.pane_read_spec(&deliver_target))?;
            pane_text_from_read_output(settings.selected, &output.stdout)?
        } else {
            pane_text.clone()
        };
        let operations = delivery_plan(adapter.as_ref(), &deliver_target, &live_text, &prompt);
        print_operations(&operations);
        if exec {
            for spec in operations
                .into_iter()
                .filter_map(|operation| operation.command)
            {
                let _ = run_command(&mut runner, &spec)?;
            }
        }
    }
    Ok(())
}

fn delivery_targets(home: &Path, target: &str, slot: &str, role: &str) -> Result<Vec<String>> {
    if !target.is_empty() {
        return Ok(vec![target.to_string()]);
    }
    let state = read_state(home)?;
    let entry = state
        .get(slot)
        .ok_or_else(|| MaehError::CacheMiss(slot.to_string()))?;
    let mut targets = Vec::new();
    if matches!(role, "primary" | "both") {
        targets.push(
            entry
                .get("primary_pane")
                .cloned()
                .ok_or_else(|| MaehError::CacheMiss(format!("{slot}:primary_pane")))?,
        );
    }
    if matches!(role, "critic" | "both") {
        targets.push(
            entry
                .get("critic_pane")
                .cloned()
                .ok_or_else(|| MaehError::CacheMiss(format!("{slot}:critic_pane")))?,
        );
    }
    if targets.is_empty() {
        return Err(MaehError::Usage(format!("unknown role {role}")));
    }
    Ok(targets)
}

pub(crate) fn verify_command(home: &Path, args: &mut Vec<String>) -> Result<()> {
    match take_arg(args, "verify command")?.as_str() {
        "prompt" => {
            let before = read_text_flag(args, "--before", "--before-file")?;
            let after = read_text_flag(args, "--after", "--after-file")?;
            let prompt = read_text_flag(args, "--prompt", "--prompt-file")?;
            let verification = verify_prompt_execution(&before, &after, &prompt)?;
            println!("maeh verify prompt");
            println!("  changed: {}", verification.changed);
            println!("  submitted: {}", verification.submitted);
            println!("  prompt head: {}", verification.prompt_head);
            Ok(())
        }
        "slot" => {
            let json = flag_present(args, "--json");
            verify_slot(home, &take_arg(args, "slot")?, json)
        }
        other => Err(MaehError::Usage(format!("unknown verify command {other}"))),
    }
}

pub(crate) fn prompt_text(args: &mut Vec<String>) -> Result<String> {
    if args.iter().any(|arg| arg == "--prompt-file") {
        return Ok(fs::read_to_string(flag_value(args, "--prompt-file", "")?)?);
    }
    flag_value(args, "--prompt", "")
}

fn pane_text(args: &mut Vec<String>) -> Result<String> {
    if args.iter().any(|arg| arg == "--pane-file") {
        return Ok(fs::read_to_string(flag_value(args, "--pane-file", "")?)?);
    }
    flag_value(args, "--pane-text", "")
}

fn read_text_flag(args: &mut Vec<String>, value_flag: &str, file_flag: &str) -> Result<String> {
    if args.iter().any(|arg| arg == file_flag) {
        return Ok(fs::read_to_string(flag_value(args, file_flag, "")?)?);
    }
    required_flag(args, value_flag)
}

#[derive(Serialize)]
struct SlotVerification {
    slot: String,
    status: String,
    worktree: String,
    primary_pane: String,
    critic_pane: String,
}

pub(crate) fn verify_slot(home: &Path, slot: &str, json: bool) -> Result<()> {
    let state = read_state(home)?;
    let entry = state
        .get(slot)
        .ok_or_else(|| MaehError::CacheMiss(slot.to_string()))?;
    for key in ["worktree", "primary_pane", "critic_pane"] {
        if entry.get(key).is_none_or(String::is_empty) {
            return Err(MaehError::CacheMiss(format!("{slot}:{key}")));
        }
    }
    let verification = SlotVerification {
        slot: slot.to_string(),
        status: entry
            .get("status")
            .cloned()
            .unwrap_or_else(|| "none".to_string()),
        worktree: entry.get("worktree").cloned().unwrap_or_default(),
        primary_pane: entry.get("primary_pane").cloned().unwrap_or_default(),
        critic_pane: entry.get("critic_pane").cloned().unwrap_or_default(),
    };
    if json {
        println!("{}", serde_json::to_string(&verification)?);
        return Ok(());
    }
    println!("slot verified");
    println!("  slot: {}", verification.slot);
    println!("  status: {}", verification.status);
    println!("  worktree: {}", verification.worktree);
    println!("  primary pane: {}", verification.primary_pane);
    println!("  critic pane: {}", verification.critic_pane);
    Ok(())
}
