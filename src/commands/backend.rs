use std::fs;
use std::path::Path;

use maeh::backend::{
    adapter_for, print_operations, print_slots, BackendAdapter, BackendSlot, RealRunner,
    ReconciliationService,
};

use crate::commands::slot::{print_backend_task_slots, print_task_slot_rows, print_worktree_rows};
use crate::commands::state::read_state;
use crate::config::{backend_settings, print_backend_resolution};
use crate::error::{MaehError, Result};
use crate::util::{flag_present, flag_value, now_epoch, take_arg};

pub(crate) fn backend_command(home: &Path, args: &mut Vec<String>) -> Result<()> {
    let command = take_arg(args, "backend command")?;
    let fixture = flag_value(args, "--fixture", "")?;
    let exec = flag_present(args, "--exec");
    if exec && !fixture.is_empty() {
        return Err(MaehError::Usage(
            "--fixture and --exec are mutually exclusive".to_string(),
        ));
    }
    let settings = backend_settings(home)?;
    let adapter = adapter_for(&settings);
    let service = ReconciliationService::new(adapter.as_ref());
    println!("maeh backend {command}");
    print_backend_resolution(&settings);
    match command.as_str() {
        "plan" => print_operations(&service.discovery_plan()),
        "discover" => {
            if fixture.is_empty() && !exec {
                print_operations(&service.discovery_plan());
            } else {
                let slots = backend_discover(home, adapter.as_ref(), &service, &fixture)?;
                print_slots(&slots);
            }
        }
        "reconcile" => {
            if fixture.is_empty() && !exec {
                print_operations(&service.discovery_plan());
            } else {
                let slots = backend_discover(home, adapter.as_ref(), &service, &fixture)?;
                let operations = service.reconcile(&read_state(home)?, &slots);
                print_operations(&operations);
            }
        }
        "list-task-slots" => {
            if fixture.is_empty() && !exec {
                print_task_slot_rows(home)?;
            } else {
                let slots = backend_discover(home, adapter.as_ref(), &service, &fixture)?;
                print_backend_task_slots(&slots);
            }
        }
        "list-worktrees" => print_worktree_rows(home)?,
        other => return Err(MaehError::Usage(format!("unknown backend command {other}"))),
    }
    Ok(())
}

fn backend_discover(
    home: &Path,
    adapter: &dyn BackendAdapter,
    service: &ReconciliationService<'_>,
    fixture: &str,
) -> Result<Vec<BackendSlot>> {
    let state = read_state(home)?;
    if !fixture.is_empty() {
        let raw = fs::read_to_string(fixture)?;
        return Ok(adapter.parse_discovery(&raw, &state, now_epoch())?);
    }
    let mut runner = RealRunner;
    Ok(service.discover_with_runner(&mut runner, &state, now_epoch())?)
}
