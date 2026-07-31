use std::collections::BTreeMap;
use std::path::Path;

use crate::commands::slot::{
    print_slot_rows, print_worktree_rows, slot_arg, slot_class, slot_close, slot_inspect,
    slot_mark, slot_nudge,
};
use crate::commands::state::read_state;
use crate::config::read_config;
use crate::error::{MaehError, Result};
use crate::util::{flag_value, now_epoch, take_arg};

pub(crate) fn cleanup_command(home: &Path, args: &mut Vec<String>) -> Result<()> {
    match take_arg(args, "cleanup command")?.as_str() {
        "list" => print_slot_rows(
            home,
            &flag_value(args, "--class", "done")?,
            &flag_value(args, "--status", "")?,
            0,
        ),
        "inspect" => {
            let slot = slot_arg(args)?;
            slot_inspect(home, &slot)
        }
        "close" => slot_close(home, args),
        "summary" => cleanup_summary(home),
        other => Err(MaehError::Usage(format!("unknown cleanup command {other}"))),
    }
}

pub(crate) fn cleanup_summary(home: &Path) -> Result<()> {
    let state = read_state(home)?;
    let mut counts = BTreeMap::<String, u64>::new();
    for entry in state.values() {
        *counts.entry(slot_class(entry, now_epoch())).or_default() += 1;
    }
    for (class, count) in counts {
        println!("{class}\t{count}");
    }
    Ok(())
}

pub(crate) fn revamp_command(home: &Path, args: &mut Vec<String>) -> Result<()> {
    match take_arg(args, "revamp command")?.as_str() {
        "list" => print_slot_rows(
            home,
            &flag_value(args, "--class", "stale")?,
            &flag_value(args, "--status", "")?,
            86_400,
        ),
        "inspect" => {
            let slot = slot_arg(args)?;
            slot_inspect(home, &slot)
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
        "nudge" => slot_nudge(home, args),
        "summary" => cleanup_summary(home),
        other => Err(MaehError::Usage(format!("unknown revamp command {other}"))),
    }
}

pub(crate) fn status_command(home: &Path, args: &mut Vec<String>) -> Result<()> {
    match take_arg(args, "status command")?.as_str() {
        "list" => print_slot_rows(
            home,
            &flag_value(args, "--class", "all")?,
            &flag_value(args, "--status", "")?,
            0,
        ),
        "inspect" => {
            let slot = slot_arg(args)?;
            slot_inspect(home, &slot)
        }
        "worktrees" => print_worktree_rows(home),
        other => Err(MaehError::Usage(format!("unknown status command {other}"))),
    }
}

pub(crate) fn cap_command(home: &Path, args: &mut Vec<String>) -> Result<()> {
    match take_arg(args, "cap command")?.as_str() {
        "check" => cap_check(home),
        other => Err(MaehError::Usage(format!("unknown cap command {other}"))),
    }
}

fn cap_check(home: &Path) -> Result<()> {
    let config = read_config(home)?;
    let mut work = 0;
    let mut review = 0;
    for entry in read_state(home)?.into_values() {
        match entry.get("status").map(String::as_str) {
            Some("active" | "blocked" | "snoozed") => work += 1,
            Some("review") => review += 1,
            _ => {}
        }
    }
    println!("cap check");
    println!("  work: {work}/{}", config.limits.context_switch_cap);
    println!("  review: {review}/{}", config.limits.review_cap);
    println!(
        "  work available: {}",
        work < config.limits.context_switch_cap
    );
    println!("  review available: {}", review < config.limits.review_cap);
    Ok(())
}
