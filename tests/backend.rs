use std::collections::BTreeMap;
use std::path::PathBuf;

use maeh::backend::*;

#[derive(Default)]
struct FakeRunner {
    outputs: Vec<CommandOutput>,
    specs: Vec<CommandSpec>,
}

impl CommandRunner for FakeRunner {
    fn run(&mut self, spec: &CommandSpec) -> Result<CommandOutput, BackendError> {
        self.specs.push(spec.clone());
        if self.outputs.is_empty() {
            Ok(CommandOutput {
                stdout: String::new(),
                stderr: String::new(),
                status: 0,
            })
        } else {
            Ok(self.outputs.remove(0))
        }
    }
}

fn tmux() -> TmuxBackend {
    TmuxBackend::new("tmuxx".to_string(), "orch".to_string())
}

fn worktree_request(include_editor: bool, focus: bool, create: bool) -> WorktreeRequest {
    WorktreeRequest {
        slot: "slot-a".to_string(),
        repo: PathBuf::from("/repo"),
        branch: "ha-feat-live".to_string(),
        base: "main".to_string(),
        path: PathBuf::from("/repo/.worktrees/live"),
        label: "live".to_string(),
        create,
        layout: LayoutOptions {
            include_editor,
            focus,
        },
    }
}

fn spawn_request(include_editor: bool, focus: bool, create: bool) -> SpawnRequest {
    SpawnRequest {
        worktree: worktree_request(include_editor, focus, create),
        task_url: "https://task".to_string(),
        primary_agent_cmd: vec!["codex".to_string(), "--primary".to_string()],
        critic_agent_cmd: vec!["codex".to_string(), "--critic".to_string()],
        editor_cmd: vec!["vi".to_string()],
    }
}

fn output(stdout: &str) -> CommandOutput {
    CommandOutput {
        stdout: stdout.to_string(),
        stderr: String::new(),
        status: 0,
    }
}

fn failing_output() -> CommandOutput {
    CommandOutput {
        stdout: String::new(),
        stderr: "fail".to_string(),
        status: 9,
    }
}

#[test]
fn backend_kind_settings_and_env_are_12_factor() {
    let env = BackendEnv {
        backend_override: None,
        herdr_session: true,
        herdr_bin: Some("h".to_string()),
        tmux_bin: Some("t".to_string()),
        tmux_session: Some("s".to_string()),
    };
    let resolved = BackendSettings::resolve(BackendKind::Auto, "herdr", "tmux", "maeh", &env);
    assert_eq!(resolved.selected, BackendKind::Herdr);
    assert_eq!(resolved.herdr_bin, "h");
    assert_eq!(resolved.tmux_bin, "t");
    assert_eq!(resolved.tmux_session, "s");
    assert!("wat".parse::<BackendKind>().is_err());

    std::env::set_var("MAEH_BACKEND", "tmux");
    std::env::set_var("MAEH_HERDR_BIN", "env-herdr");
    std::env::set_var("MAEH_TMUX_BIN", "env-tmux");
    std::env::set_var("MAEH_TMUX_SESSION", "env-session");
    let from_env = BackendEnv::from_env().unwrap();
    assert_eq!(from_env.backend_override, Some(BackendKind::Tmux));
    assert_eq!(from_env.herdr_bin.as_deref(), Some("env-herdr"));
    assert_eq!(from_env.tmux_bin.as_deref(), Some("env-tmux"));
    assert_eq!(from_env.tmux_session.as_deref(), Some("env-session"));
    std::env::remove_var("MAEH_BACKEND");
    std::env::remove_var("MAEH_HERDR_BIN");
    std::env::remove_var("MAEH_TMUX_BIN");
    std::env::remove_var("MAEH_TMUX_SESSION");

    assert_eq!("auto".parse::<BackendKind>().unwrap(), BackendKind::Auto);
    assert_eq!("herdr".parse::<BackendKind>().unwrap(), BackendKind::Herdr);
    assert_eq!("tmux".parse::<BackendKind>().unwrap(), BackendKind::Tmux);
    assert_eq!(
        LayoutOptions::default(),
        LayoutOptions {
            include_editor: true,
            focus: false
        }
    );
}

#[test]
fn real_runner_supports_cwd_and_env_without_touching_backends() {
    let mut spec = CommandSpec::new("pwd", &[]);
    spec.cwd = Some(PathBuf::from("/"));
    spec.env.insert("MAEH_TEST".to_string(), "1".to_string());
    let mut runner = RealRunner;
    let output = runner.run(&spec).unwrap();
    assert_eq!(output.status, 0);
    assert_eq!(output.stdout.trim(), "/");
}

#[test]
fn fake_runner_asserts_discovery_argv_env_cwd_and_parsing() {
    let adapter = tmux();
    let mut runner = FakeRunner {
        outputs: vec![output(
            "orch:1\u{1f}90\u{1f}task-a\u{1f}https://task\u{1f}active\u{1f}0\u{1f}/tmp/wt\n",
        )],
        specs: Vec::new(),
    };
    let slots = ReconciliationService::new(&adapter)
        .discover_with_runner(&mut runner, &SlotState::new(), 100)
        .unwrap();
    assert_eq!(runner.specs[0].program, "tmuxx");
    assert_eq!(runner.specs[0].args[0], "list-windows");
    assert_eq!(runner.specs[0].args[1], "-t");
    assert_eq!(runner.specs[0].args[2], "orch");
    assert!(runner.specs[0].cwd.is_none());
    assert!(runner.specs[0].env.is_empty());
    assert_eq!(slots[0].slot, "orch:1");
    assert_eq!(slots[0].age_secs, 10);
    assert_eq!(slots[0].primary_pane, "");
    assert_eq!(slots[0].critic_pane, "");

    let herdr = HerdrBackend::new("herdrx".to_string());
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
        outputs: vec![output(
            r#"{"result":{"snapshot":{"workspaces":[{"workspace_id":"w1","label":"slot","worktree":{"checkout_path":"/tmp/wt"}}],"panes":[{"workspace_id":"w1","pane_id":"w1:p2","agent":"primary"}]}}}"#,
        )],
        specs: Vec::new(),
    };
    let slots = ReconciliationService::new(&herdr)
        .discover_with_runner(&mut runner, &state, 100)
        .unwrap();
    assert_eq!(runner.specs[0].args, ["api", "snapshot"]);
    assert_eq!(slots[0].slot, "w1");
    assert_eq!(slots[0].age_secs, 5);
    assert_eq!(slots[0].primary_pane, "w1:p2");
    assert_eq!(
        ReconciliationService::new(&herdr).discovery_plan()[0].target,
        "herdr"
    );
}

#[test]
fn worktree_plan_and_spawn_plan_cover_herdr_and_tmux_no_editor() {
    let herdr = HerdrBackend::new("herdrx".to_string());
    let request = worktree_request(false, false, true);
    let plan = herdr.worktree_plan(&request);
    assert_eq!(plan[0].action, "worktree-create");
    assert_eq!(
        plan[0].command.as_ref().unwrap().argv(),
        [
            "herdrx",
            "worktree",
            "create",
            "--cwd",
            "/repo",
            "--branch",
            "ha-feat-live",
            "--base",
            "main",
            "--path",
            "/repo/.worktrees/live",
            "--label",
            "live",
            "--no-focus",
            "--json",
        ]
    );
    let open_plan = herdr.worktree_plan(&worktree_request(true, true, false));
    assert_eq!(open_plan[0].action, "worktree-open");
    assert!(open_plan[0]
        .command
        .as_ref()
        .unwrap()
        .args
        .contains(&"--focus".to_string()));

    let spawn = herdr.spawn_plan(&spawn_request(false, false, true));
    assert_eq!(
        spawn.iter().filter(|op| op.action == "editor-pane").count(),
        0
    );
    assert!(spawn.iter().any(|op| op.action == "primary-agent"));
    assert!(spawn.iter().any(|op| op.action == "critic-agent"));

    let tmux_spawn = tmux().spawn_plan(&spawn_request(false, false, true));
    assert_eq!(tmux_spawn[0].command.as_ref().unwrap().program, "git");
    assert!(tmux_spawn.iter().any(|op| op.action == "primary-agent"));
    assert_eq!(
        tmux_spawn
            .iter()
            .filter(|op| op.action == "editor-pane")
            .count(),
        0
    );
}

#[test]
fn tmux_worktree_spawn_editor_and_error_paths_use_real_ids() {
    let adapter = tmux();
    let request = worktree_request(true, true, true);
    let plan = adapter.worktree_plan(&request);
    assert_eq!(plan[0].action, "worktree-create");
    assert_eq!(plan[1].action, "workspace-open");
    assert!(plan[1]
        .command
        .as_ref()
        .unwrap()
        .args
        .contains(&"live".to_string()));
    assert_eq!(
        adapter.pane_read_spec("%1").argv(),
        ["tmuxx", "capture-pane", "-p", "-t", "%1", "-S", "-80"]
    );
    assert!(adapter
        .delivery_specs(
            "%1",
            &DeliveryIntent::Noop {
                reason: "busy".to_string()
            }
        )
        .is_empty());

    let mut runner = FakeRunner {
        outputs: vec![output(""), output("@2\t%2\n")],
        specs: Vec::new(),
    };
    let record = adapter.execute_worktree(&mut runner, &request).unwrap();
    assert_eq!(record.window_id, "@2");
    assert_eq!(record.workspace_id, "@2");

    let editor_spawn = spawn_request(true, true, true);
    let editor_plan = adapter.spawn_plan(&editor_spawn);
    assert!(editor_plan.iter().any(|op| op.action == "editor-pane"));
    let mut runner = FakeRunner {
        outputs: vec![
            output(""),
            output("@9\t%1\n"),
            output("%2\n"),
            output("%3\n"),
        ],
        specs: Vec::new(),
    };
    let spawn = adapter.execute_spawn(&mut runner, &editor_spawn).unwrap();
    assert_eq!(spawn.editor_pane, "%2");
    assert_eq!(spawn.critic_pane, "%3");

    let mut failing = FakeRunner {
        outputs: vec![failing_output()],
        specs: Vec::new(),
    };
    assert!(adapter.execute_worktree(&mut failing, &request).is_err());
    let mut bad_parse = FakeRunner {
        outputs: vec![output("no tabs\n")],
        specs: Vec::new(),
    };
    assert!(adapter
        .execute_worktree(&mut bad_parse, &worktree_request(false, false, false))
        .is_err());
}

#[test]
fn herdr_editor_empty_path_and_error_paths_are_deterministic() {
    let adapter = HerdrBackend::new("herdrx".to_string());
    let mut request = worktree_request(true, true, false);
    request.path = PathBuf::new();
    let plan = adapter.worktree_plan(&request);
    assert_eq!(plan[0].action, "worktree-open");
    assert!(plan[0]
        .command
        .as_ref()
        .unwrap()
        .args
        .windows(2)
        .any(|pair| pair == ["--branch", "ha-feat-live"]));
    let empty_path_spawn = SpawnRequest {
        worktree: request.clone(),
        task_url: "https://task".to_string(),
        primary_agent_cmd: vec!["codex".to_string()],
        critic_agent_cmd: vec!["codex".to_string()],
        editor_cmd: vec![],
    };
    let empty_path_plan = adapter.spawn_plan(&empty_path_spawn);
    assert!(empty_path_plan.iter().any(|op| op
        .command
        .as_ref()
        .is_some_and(|spec| spec.args.contains(&"$worktree".to_string()))));
    assert_eq!(
        adapter.pane_read_spec("w1:p2").argv(),
        [
            "herdrx",
            "agent",
            "read",
            "w1:p2",
            "--source",
            "recent-unwrapped",
            "--lines",
            "80",
            "--format",
            "text"
        ]
    );
    assert!(adapter
        .delivery_specs(
            "w1:p2",
            &DeliveryIntent::Noop {
                reason: "busy".to_string()
            }
        )
        .is_empty());

    let editor_spawn = spawn_request(true, false, true);
    let editor_plan = adapter.spawn_plan(&editor_spawn);
    assert!(editor_plan.iter().any(|op| op.action == "editor-pane"));
    let mut runner = FakeRunner {
        outputs: vec![
            output(r#"{"result":{"workspace_id":"w9"}}"#),
            output(r#"{"result":{"pane_id":"w9:p1"}}"#),
            output(r#"{"result":{"id":"w9:p2"}}"#),
            output(r#"{"result":{"terminal_id":"w9:p3"}}"#),
        ],
        specs: Vec::new(),
    };
    let spawn = adapter.execute_spawn(&mut runner, &editor_spawn).unwrap();
    assert_eq!(spawn.editor_pane, "w9:p1");
    assert_eq!(spawn.primary_pane, "w9:p2");
    assert_eq!(spawn.critic_pane, "w9:p3");
    assert_eq!(spawn.worktree.worktree, "/repo/.worktrees/live");

    let mut array_id = FakeRunner {
        outputs: vec![output(r#"[{"workspace_id":"w-array","path":"/array-wt"}]"#)],
        specs: Vec::new(),
    };
    let array_record = adapter
        .execute_worktree(&mut array_id, &worktree_request(false, false, true))
        .unwrap();
    assert_eq!(array_record.workspace_id, "w-array");
    let mut missing_id = FakeRunner {
        outputs: vec![output(r#"{"result":{"path":"/tmp/wt"}}"#)],
        specs: Vec::new(),
    };
    assert!(adapter
        .execute_worktree(&mut missing_id, &worktree_request(false, false, true))
        .is_err());
    let mut array_missing_id = FakeRunner {
        outputs: vec![output(r#"[{"path":"/tmp/wt"}]"#)],
        specs: Vec::new(),
    };
    assert!(adapter
        .execute_worktree(&mut array_missing_id, &worktree_request(false, false, true))
        .is_err());
    let mut primitive = FakeRunner {
        outputs: vec![output("7")],
        specs: Vec::new(),
    };
    assert!(adapter
        .execute_worktree(&mut primitive, &worktree_request(false, false, true))
        .is_err());
    let mut failing = FakeRunner {
        outputs: vec![failing_output()],
        specs: Vec::new(),
    };
    assert!(adapter
        .execute_worktree(&mut failing, &worktree_request(false, false, true))
        .is_err());
}

#[test]
fn execute_worktree_and_spawn_persist_real_backend_ids() {
    let herdr = HerdrBackend::new("herdrx".to_string());
    let mut runner = FakeRunner {
        outputs: vec![output(
            r#"{"result":{"workspace_id":"w9","path":"/repo/.worktrees/live"}}"#,
        )],
        specs: Vec::new(),
    };
    let record = herdr
        .execute_worktree(&mut runner, &worktree_request(false, false, true))
        .unwrap();
    assert_eq!(record.workspace_id, "w9");
    assert_eq!(record.worktree, "/repo/.worktrees/live");
    assert_eq!(runner.specs[0].program, "herdrx");

    let mut runner = FakeRunner {
        outputs: vec![
            output(r#"{"result":{"workspace_id":"w9","path":"/repo/.worktrees/live"}}"#),
            output(r#"{"result":{"pane_id":"w9:p2"}}"#),
            output(r#"{"result":{"pane_id":"w9:p3"}}"#),
        ],
        specs: Vec::new(),
    };
    let spawn = herdr
        .execute_spawn(&mut runner, &spawn_request(false, false, true))
        .unwrap();
    assert_eq!(spawn.primary_pane, "w9:p2");
    assert_eq!(spawn.critic_pane, "w9:p3");
    assert_eq!(spawn.editor_pane, "");

    let mut tmux_runner = FakeRunner {
        outputs: vec![output(""), output("@9\t%1\n"), output("%3\n")],
        specs: Vec::new(),
    };
    let spawn = tmux()
        .execute_spawn(&mut tmux_runner, &spawn_request(false, false, true))
        .unwrap();
    assert_eq!(spawn.worktree.window_id, "@9");
    assert_eq!(spawn.primary_pane, "%1");
    assert_eq!(spawn.critic_pane, "%3");
    assert_eq!(tmux_runner.specs[0].program, "git");
    assert_eq!(tmux_runner.specs[1].args[0], "new-window");
    assert_eq!(tmux_runner.specs[2].args[0], "split-window");
}

#[test]
fn prompt_delivery_policy_and_adapter_commands_are_explicit_text_plus_enter() {
    let prompt = "Do the task";
    assert_eq!(
        delivery_intent("ready\n› ", prompt),
        DeliveryIntent::SubmitQueued {
            text: prompt.to_string()
        }
    );
    assert_eq!(
        delivery_intent("Do you trust this folder?", prompt),
        DeliveryIntent::AnswerBlocker {
            blocker: HarnessBlocker::Trust,
            response: "1".to_string()
        }
    );
    assert_eq!(
        delivery_intent("Trust this folder", prompt),
        DeliveryIntent::AnswerBlocker {
            blocker: HarnessBlocker::Trust,
            response: "1".to_string()
        }
    );
    assert_eq!(
        delivery_intent("Update available. Install?", prompt),
        DeliveryIntent::AnswerBlocker {
            blocker: HarnessBlocker::Update,
            response: "n".to_string()
        }
    );
    assert_eq!(
        delivery_intent("Update: new version is ready", prompt),
        DeliveryIntent::AnswerBlocker {
            blocker: HarnessBlocker::Update,
            response: "n".to_string()
        }
    );
    assert_eq!(
        delivery_intent("Press Enter to continue", prompt),
        DeliveryIntent::AnswerBlocker {
            blocker: HarnessBlocker::Continue,
            response: String::new()
        }
    );
    assert_eq!(
        delivery_intent("Continue?", prompt),
        DeliveryIntent::AnswerBlocker {
            blocker: HarnessBlocker::Continue,
            response: String::new()
        }
    );
    assert_eq!(
        delivery_intent("working...", prompt),
        DeliveryIntent::Noop {
            reason: "pane busy or unknown".to_string()
        }
    );
    assert_eq!(
        delivery_intent("", prompt),
        DeliveryIntent::Noop {
            reason: "pane busy or unknown".to_string()
        }
    );
    assert_eq!(
        delivery_intent("ready\n› ", ""),
        DeliveryIntent::Noop {
            reason: "no queued prompt".to_string()
        }
    );
    assert_eq!(
        DeliveryIntent::SubmitQueued {
            text: prompt.to_string()
        }
        .action(),
        "submit-queued"
    );
    assert_eq!(
        DeliveryIntent::AnswerBlocker {
            blocker: HarnessBlocker::Trust,
            response: "1".to_string()
        }
        .action(),
        "unblock-trust"
    );

    let herdr = HerdrBackend::new("herdrx".to_string());
    let operations = delivery_plan(&herdr, "w1:p2", "ready\n› ", prompt);
    assert_eq!(operations.len(), 2);
    assert_eq!(operations[0].action, "send-text");
    assert_eq!(
        operations[0].command.as_ref().unwrap().argv(),
        ["herdrx", "agent", "send", "w1:p2", "Do the task"]
    );
    assert_eq!(operations[1].action, "submit-enter");
    assert_eq!(
        operations[1].command.as_ref().unwrap().argv(),
        ["herdrx", "pane", "send-keys", "w1:p2", "Enter"]
    );

    let tmux_operations = delivery_plan(&tmux(), "%1", "ready\n> ", prompt);
    assert_eq!(
        tmux_operations[0].command.as_ref().unwrap().argv(),
        ["tmuxx", "send-keys", "-t", "%1", "-l", "Do the task"]
    );
    assert_eq!(
        tmux_operations[1].command.as_ref().unwrap().argv(),
        ["tmuxx", "send-keys", "-t", "%1", "Enter"]
    );
    let tmux_trust = delivery_plan(&tmux(), "%1", "Trust this folder", prompt);
    assert_eq!(
        tmux_trust[0].command.as_ref().unwrap().argv(),
        ["tmuxx", "send-keys", "-t", "%1", "-l", "1"]
    );

    let continue_ops = delivery_plan(&herdr, "w1:p2", "Press Enter to continue", prompt);
    assert_eq!(continue_ops.len(), 1);
    assert_eq!(continue_ops[0].action, "submit-enter");
    let noop_ops = delivery_plan(&herdr, "w1:p2", "working...", prompt);
    assert_eq!(noop_ops[0].kind, OperationKind::Read);
    assert!(noop_ops[0].command.is_none());
}

#[test]
fn pane_read_output_parsing_matches_live_backends() {
    assert_eq!(
        pane_text_from_read_output(
            BackendKind::Herdr,
            r#"{"result":{"read":{"text":"ready\n› "}}}"#,
        )
        .unwrap(),
        "ready\n› "
    );
    assert!(pane_text_from_read_output(BackendKind::Herdr, r#"{"result":{}}"#).is_err());
    assert_eq!(
        pane_text_from_read_output(BackendKind::Tmux, "ready\n> ").unwrap(),
        "ready\n> "
    );
    assert_eq!(
        pane_text_from_read_output(BackendKind::Auto, "raw").unwrap(),
        "raw"
    );
}

#[test]
fn verify_prompt_execution_requires_changed_submitted_output() {
    let ok = verify_prompt_execution("› Do the task", "Working on it", "Do the task").unwrap();
    assert!(ok.changed);
    assert!(ok.submitted);
    assert_eq!(ok.prompt_head, "Do the task");
    assert!(verify_prompt_execution("same", "same", "Do the task").is_err());
    assert!(verify_prompt_execution("› Do the task", "› Do the task", "Do the task").is_err());
}

#[test]
fn reconcile_and_parser_error_paths_are_deterministic() {
    let adapter = tmux();
    let mut runner = FakeRunner {
        outputs: vec![CommandOutput {
            stdout: String::new(),
            stderr: "nope".to_string(),
            status: 7,
        }],
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
    assert!(adapter
        .parse_discovery("bad\n", &SlotState::new(), 1)
        .is_err());
    assert_eq!(
        adapter
            .parse_discovery(
                "orch:1\u{1f}bad\u{1f}slot\u{1f}\u{1f}\u{1f}\u{1f}/tmp/wt\n",
                &SlotState::new(),
                1,
            )
            .unwrap(),
        Vec::new()
    );
    let slots = adapter
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
    assert!(herdr
        .parse_discovery(r#"{"workspaces":{}}"#, &SlotState::new(), 1)
        .is_err());
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
                1
            )
            .unwrap(),
        Vec::new()
    );
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
    state.insert(
        "w3".to_string(),
        BTreeMap::from([
            ("task_url".to_string(), "https://three".to_string()),
            ("snooze_until".to_string(), "42".to_string()),
        ]),
    );
    assert_eq!(
        herdr
            .parse_discovery(
                r#"{"snapshot":{"workspaces":[{"workspace_id":"w3","worktree":{"checkout_path":"/tmp/live-wt"}}]}}"#,
                &state,
                10,
            )
            .unwrap()[0]
            .worktree,
        "/tmp/live-wt"
    );
    state.insert(
        "w4".to_string(),
        BTreeMap::from([("task_url".to_string(), "https://four".to_string())]),
    );
    assert_eq!(
        herdr
            .parse_discovery(
                r#"{"workspaces":[{"workspace_id":"w4","worktree":{}}]}"#,
                &state,
                10
            )
            .unwrap()[0]
            .worktree,
        ""
    );
}

#[test]
fn reconcile_print_and_selection_helpers_cover_remaining_paths() {
    let adapter = tmux();
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
    state.insert(
        "orch:2".to_string(),
        BTreeMap::from([("task_url".to_string(), "https://missing".to_string())]),
    );
    let discovered = vec![BackendSlot {
        backend: BackendKind::Tmux,
        slot: "orch:1".to_string(),
        task_url: "https://actual".to_string(),
        status: "active".to_string(),
        snooze_until: "0".to_string(),
        age_secs: 3,
        name: "slot".to_string(),
        worktree: "/tmp/wt".to_string(),
        primary_pane: "".to_string(),
        critic_pane: "".to_string(),
    }];
    assert_eq!(
        service.reconcile(&state, &discovered)[0].action,
        "metadata-drift"
    );
    assert_eq!(
        service.reconcile(&SlotState::new(), &discovered)[0].action,
        "untracked-live-slot"
    );
    state.insert(
        "orch:1".to_string(),
        BTreeMap::from([("task_url".to_string(), "https://actual".to_string())]),
    );
    let ops = service.reconcile(&state, &discovered);
    assert_eq!(ops[0].action, "ok");
    assert_eq!(ops[1].action, "missing-live-slot");

    let mut command = CommandSpec::new("cmd", &["a"]);
    command.cwd = Some(PathBuf::from("/tmp"));
    command.env.insert("K".to_string(), "V".to_string());
    assert_eq!(command.argv(), ["cmd", "a"]);
    print_operations(&[OperationPlan::read(
        "inspect",
        "target",
        "detail".to_string(),
        Some(command),
    )]);
    print_slots(&[discovered[0].clone()]);

    let settings = BackendSettings {
        requested: BackendKind::Auto,
        selected: BackendKind::Auto,
        herdr_bin: "herdr".to_string(),
        tmux_bin: "tmux".to_string(),
        tmux_session: "maeh".to_string(),
    };
    assert!(std::panic::catch_unwind(|| adapter_for(&settings)).is_err());
}
