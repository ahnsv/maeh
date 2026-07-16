use std::collections::BTreeMap;
use std::path::PathBuf;

use maeh::backend::*;

#[derive(Default)]
struct FakeRunner {
    output: Option<CommandOutput>,
    specs: Vec<CommandSpec>,
}

impl CommandRunner for FakeRunner {
    fn run(&mut self, spec: &CommandSpec) -> Result<CommandOutput, BackendError> {
        self.specs.push(spec.clone());
        Ok(self.output.take().unwrap_or(CommandOutput {
            stdout: String::new(),
            stderr: String::new(),
            status: 0,
        }))
    }
}

#[test]
fn backend_kind_resolves_from_config_and_env() {
    let env = BackendEnv {
        backend_override: None,
        herdr_session: true,
        herdr_bin: Some("h".to_string()),
        tmux_bin: Some("t".to_string()),
    };
    let resolved = BackendSettings::resolve(BackendKind::Auto, "herdr", "tmux", &env);
    assert_eq!(resolved.selected, BackendKind::Herdr);
    assert_eq!(resolved.herdr_bin, "h");
    assert_eq!(resolved.tmux_bin, "t");
    assert!("wat".parse::<BackendKind>().is_err());
    std::env::set_var("MAEH_HERDR_BIN", "env-herdr");
    let from_env = BackendEnv::from_env().unwrap();
    assert_eq!(from_env.herdr_bin.as_deref(), Some("env-herdr"));
    std::env::remove_var("MAEH_HERDR_BIN");
    assert_eq!("auto".parse::<BackendKind>().unwrap(), BackendKind::Auto);
    assert_eq!("herdr".parse::<BackendKind>().unwrap(), BackendKind::Herdr);
    assert_eq!("tmux".parse::<BackendKind>().unwrap(), BackendKind::Tmux);
}

#[test]
fn herdr_adapter_exposes_discovery_spec_directly() {
    let adapter = HerdrBackend::new("herdrx".to_string());
    assert_eq!(adapter.kind(), BackendKind::Herdr);
    let spec = adapter.discovery_spec();
    assert_eq!(spec.program, "herdrx");
    assert_eq!(spec.args, ["api", "snapshot"]);
}

#[test]
fn fake_runner_asserts_tmux_discovery_argv_env_cwd_and_parsing() {
    let adapter = TmuxBackend::new("tmuxx".to_string());
    let mut runner = FakeRunner {
        output: Some(CommandOutput {
            stdout:
                "orch:1\u{1f}90\u{1f}task-a\u{1f}https://task\u{1f}active\u{1f}0\u{1f}/tmp/wt\n"
                    .to_string(),
            stderr: String::new(),
            status: 0,
        }),
        specs: Vec::new(),
    };
    let slots = ReconciliationService::new(&adapter)
        .discover_with_runner(&mut runner, &SlotState::new(), 100)
        .unwrap();
    assert_eq!(runner.specs[0].program, "tmuxx");
    assert_eq!(runner.specs[0].args[0], "list-windows");
    assert!(runner.specs[0].cwd.is_none());
    assert!(runner.specs[0].env.is_empty());
    assert_eq!(slots[0].slot, "orch:1");
    assert_eq!(slots[0].age_secs, 10);
    assert_eq!(slots[0].primary_pane, "orch:1.1");
}

#[test]
fn fake_runner_asserts_herdr_discovery_argv_env_cwd_and_parsing() {
    let adapter = HerdrBackend::new("herdrx".to_string());
    let mut state = SlotState::new();
    state.insert(
        "w1".to_string(),
        BTreeMap::from([
            ("task_url".to_string(), "https://task".to_string()),
            ("status".to_string(), "active".to_string()),
            ("last_activity_epoch".to_string(), "95".to_string()),
            ("primary_pane".to_string(), "w1:p2".to_string()),
            ("critic_pane".to_string(), "w1:p3".to_string()),
        ]),
    );
    let mut runner = FakeRunner {
        output: Some(CommandOutput {
            stdout: r#"{"result":{"snapshot":{"workspaces":[{"workspace_id":"w1","label":"slot","worktree":{"checkout_path":"/tmp/wt"}}],"panes":[{"workspace_id":"w1","pane_id":"w1:p2","agent":"primary"}]}}}"#.to_string(),
            stderr: String::new(),
            status: 0,
        }),
        specs: Vec::new(),
    };
    let slots = ReconciliationService::new(&adapter)
        .discover_with_runner(&mut runner, &state, 100)
        .unwrap();
    assert_eq!(runner.specs[0].program, "herdrx");
    assert_eq!(runner.specs[0].args, ["api", "snapshot"]);
    assert!(runner.specs[0].cwd.is_none());
    assert!(runner.specs[0].env.is_empty());
    assert_eq!(slots[0].slot, "w1");
    assert_eq!(slots[0].age_secs, 5);
    assert_eq!(slots[0].primary_pane, "w1:p2");
    let plan = ReconciliationService::new(&adapter).discovery_plan();
    assert_eq!(plan[0].target, "herdr");
}

#[test]
fn nonzero_runner_status_is_a_backend_error() {
    let adapter = TmuxBackend::new("tmuxx".to_string());
    let mut runner = FakeRunner {
        output: Some(CommandOutput {
            stdout: String::new(),
            stderr: "nope".to_string(),
            status: 7,
        }),
        specs: Vec::new(),
    };
    let error = ReconciliationService::new(&adapter)
        .discover_with_runner(&mut runner, &SlotState::new(), 1)
        .unwrap_err();
    assert!(matches!(
        error,
        BackendError::CommandFailed {
            program,
            status: 7
        } if program == "tmuxx"
    ));
}

#[test]
fn reconcile_compares_normalized_records() {
    let adapter = TmuxBackend::new("tmux".to_string());
    let mut state = SlotState::new();
    state.insert(
        "orch:1".to_string(),
        BTreeMap::from([("task_url".to_string(), "https://task".to_string())]),
    );
    state.insert(
        "orch:2".to_string(),
        BTreeMap::from([("task_url".to_string(), "https://missing".to_string())]),
    );
    let discovered = vec![BackendSlot {
        backend: BackendKind::Tmux,
        slot: "orch:1".to_string(),
        task_url: "https://task".to_string(),
        status: "active".to_string(),
        snooze_until: "0".to_string(),
        age_secs: 3,
        name: "slot".to_string(),
        worktree: "/tmp/wt".to_string(),
        primary_pane: "orch:1.1".to_string(),
        critic_pane: "orch:1.2".to_string(),
    }];
    let operations = ReconciliationService::new(&adapter).reconcile(&state, &discovered);
    assert_eq!(operations[0].action, "ok");
    assert_eq!(operations[1].action, "missing-live-slot");
}

#[test]
fn parser_error_and_skip_paths_are_deterministic() {
    let tmux = TmuxBackend::new("tmux".to_string());
    assert!(tmux.parse_discovery("bad\n", &SlotState::new(), 1).is_err());
    assert_eq!(
        tmux.parse_discovery(
            "orch:1\u{1f}bad\u{1f}slot\u{1f}\u{1f}\u{1f}\u{1f}/tmp/wt\n",
            &SlotState::new(),
            1,
        )
        .unwrap(),
        Vec::new()
    );
    let slots = tmux
        .parse_discovery(
            "orch:1\u{1f}bad\u{1f}slot\u{1f}https://task\u{1f}\u{1f}\u{1f}/tmp/wt\n",
            &SlotState::new(),
            1,
        )
        .unwrap();
    assert_eq!(slots[0].status, "none");
    assert_eq!(slots[0].snooze_until, "0");

    let herdr = HerdrBackend::new("herdr".to_string());
    assert!(herdr.parse_discovery("{}", &SlotState::new(), 1).is_err());
    let skipped = herdr
        .parse_discovery(
            r#"{"workspaces":[{"workspace_id":""},{"workspace_id":"w1"}],"panes":[]}"#,
            &SlotState::new(),
            1,
        )
        .unwrap();
    assert_eq!(skipped, Vec::new());
    let mut state = SlotState::new();
    state.insert("w1".to_string(), BTreeMap::new());
    assert_eq!(
        herdr
            .parse_discovery(
                r#"{"workspaces":[{"workspace_id":"w1"}],"panes":[]}"#,
                &state,
                1,
            )
            .unwrap(),
        Vec::new()
    );
    assert!(herdr
        .parse_discovery(r#"{"workspaces":{}}"#, &SlotState::new(), 1)
        .is_err());

    state.insert(
        "w2".to_string(),
        BTreeMap::from([
            ("task_url".to_string(), "https://two".to_string()),
            ("worktree".to_string(), "/tmp/state-wt".to_string()),
        ]),
    );
    let no_panes = herdr
        .parse_discovery(
            r#"{"snapshot":{"workspaces":[{"workspace_id":"w2","label":7}],"panes":{}}}"#,
            &state,
            10,
        )
        .unwrap();
    assert_eq!(no_panes[0].status, "none");
    assert_eq!(no_panes[0].snooze_until, "0");
    assert_eq!(no_panes[0].worktree, "/tmp/state-wt");
    assert_eq!(no_panes[0].primary_pane, "");
    assert_eq!(no_panes[0].name, "");

    state.insert(
        "w3".to_string(),
        BTreeMap::from([
            ("task_url".to_string(), "https://three".to_string()),
            ("snooze_until".to_string(), "42".to_string()),
        ]),
    );
    let top_snapshot = herdr
        .parse_discovery(
            r#"{"snapshot":{"workspaces":[{"workspace_id":"w3","worktree":{"checkout_path":"/tmp/live-wt"}}]}}"#,
            &state,
            10,
        )
        .unwrap();
    assert_eq!(top_snapshot[0].snooze_until, "42");
    assert_eq!(top_snapshot[0].worktree, "/tmp/live-wt");

    state.insert(
        "w4".to_string(),
        BTreeMap::from([("task_url".to_string(), "https://four".to_string())]),
    );
    let no_worktree = herdr
        .parse_discovery(
            r#"{"workspaces":[{"workspace_id":"w4","worktree":{}}]}"#,
            &state,
            10,
        )
        .unwrap();
    assert_eq!(no_worktree[0].worktree, "");
}

#[test]
fn reconcile_reports_drift_untracked_and_empty_state() {
    let adapter = TmuxBackend::new("tmux".to_string());
    let service = ReconciliationService::new(&adapter);
    let mut state = SlotState::new();
    state.insert("empty".to_string(), BTreeMap::new());
    assert_eq!(
        service.reconcile(&state, &[])[0].detail,
        "no managed slots found"
    );
    state.insert(
        "orch:1".to_string(),
        BTreeMap::from([("task_url".to_string(), "https://expected".to_string())]),
    );
    let discovered = vec![BackendSlot {
        backend: BackendKind::Tmux,
        slot: "orch:1".to_string(),
        task_url: "https://actual".to_string(),
        status: "active".to_string(),
        snooze_until: "0".to_string(),
        age_secs: 0,
        name: String::new(),
        worktree: String::new(),
        primary_pane: String::new(),
        critic_pane: String::new(),
    }];
    assert_eq!(
        service.reconcile(&state, &discovered)[0].action,
        "metadata-drift"
    );
    assert_eq!(
        service.reconcile(&SlotState::new(), &discovered)[0].action,
        "untracked-live-slot"
    );
}

#[test]
fn runner_and_print_helpers_cover_command_shape() {
    let mut command = CommandSpec::new("/usr/bin/env", &[]);
    command.cwd = Some(PathBuf::from("/"));
    command
        .env
        .insert("MAEH_BACKEND_TEST".to_string(), "ok".to_string());
    assert_eq!(command.argv(), ["/usr/bin/env"]);
    let mut runner = RealRunner;
    let output = runner.run(&command).unwrap();
    assert_eq!(output.status, 0);
    assert!(output.stdout.contains("MAEH_BACKEND_TEST=ok"));
    print_operations(&[OperationPlan::read(
        "inspect",
        "target",
        "detail".to_string(),
        Some(command),
    )]);
    print_slots(&[BackendSlot {
        backend: BackendKind::Tmux,
        slot: "slot".to_string(),
        task_url: "url".to_string(),
        status: "status".to_string(),
        snooze_until: "0".to_string(),
        age_secs: 1,
        name: "name".to_string(),
        worktree: "wt".to_string(),
        primary_pane: "p1".to_string(),
        critic_pane: "p2".to_string(),
    }]);
}

#[test]
fn adapter_selection_rejects_unresolved_auto() {
    let settings = BackendSettings {
        requested: BackendKind::Auto,
        selected: BackendKind::Auto,
        herdr_bin: "herdr".to_string(),
        tmux_bin: "tmux".to_string(),
    };
    assert!(std::panic::catch_unwind(|| adapter_for(&settings)).is_err());
}
