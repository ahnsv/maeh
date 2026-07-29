use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::config::state_path;
use crate::error::{MaehError, Result};
use crate::util::{take_arg, write_json};

pub(crate) type State = BTreeMap<String, BTreeMap<String, String>>;

pub(crate) fn state_command(home: &Path, args: &mut Vec<String>) -> Result<()> {
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

pub(crate) fn read_state(home: &Path) -> Result<State> {
    let path = state_path(home);
    if path.exists() {
        Ok(serde_json::from_str(&fs::read_to_string(path)?)?)
    } else {
        Ok(State::new())
    }
}

pub(crate) fn write_state(home: &Path, state: &State) -> Result<()> {
    write_json(&state_path(home), &serde_json::to_value(state)?)
}

pub(crate) fn state_delete_slot(home: &Path, slot: &str) -> Result<()> {
    let mut state = read_state(home)?;
    state.remove(slot);
    write_state(home, &state)?;
    println!("state slot deleted");
    println!("  slot: {slot}");
    Ok(())
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
