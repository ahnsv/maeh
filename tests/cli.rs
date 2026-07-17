use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use assert_cmd::Command;
use chrono::{Datelike, Timelike};
use tempfile::TempDir;

fn maeh() -> Command {
    let mut cmd = Command::cargo_bin("maeh").expect("binary exists");
    cmd.env_remove("MAEH_HOME")
        .env_remove("MAEH_NOW")
        .env_remove("MAEH_EPOCH")
        .env_remove("MAEH_DEBUG")
        .env_remove("MAEH_BACKEND")
        .env_remove("MAEH_HERDR_BIN")
        .env_remove("MAEH_TMUX_BIN")
        .env_remove("MAEH_TMUX_SESSION")
        .env_remove("MAEH_INCLUDE_EDITOR")
        .env_remove("MAEH_FOCUS")
        .env_remove("MAEH_PRIMARY_AGENT_CMD")
        .env_remove("MAEH_CRITIC_AGENT_CMD")
        .env_remove("MAEH_EDITOR_CMD")
        .env_remove("HERDR_ENV")
        .env_remove("HERDR_SOCKET_PATH");
    cmd
}

fn init_home(home: &Path) {
    maeh()
        .arg("--home")
        .arg(home)
        .arg("init")
        .assert()
        .success();
}

fn register_slot(home: &Path, slot: &str, backend: &str, status: &str, worktree: &str, repo: &str) {
    maeh()
        .arg("--home")
        .arg(home)
        .args([
            "workspace",
            "register",
            "--slot",
            slot,
            "--workspace",
            &format!("workspace-{slot}"),
            "--worktree",
            worktree,
            "--repo",
            repo,
            "--task-url",
            &format!("https://tasks/{slot}"),
            "--primary-pane",
            &format!("{slot}:p"),
            "--critic-pane",
            &format!("{slot}:c"),
            "--backend",
            backend,
            "--status",
            status,
        ])
        .assert()
        .success();
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

fn local_work_time() -> (u32, u32, bool) {
    let now = chrono::Local::now();
    let dow = now.weekday().number_from_monday();
    let hour = now.hour();
    (dow, hour, (1..=5).contains(&dow) && (9..17).contains(&hour))
}

#[test]
fn help_output_is_styled_and_lists_core_commands() {
    maeh().arg("--help").assert().success().stdout(
        "Typed orchestration CLI for hmph and Herdr agents\n\nUsage: maeh [--home PATH] <command>\n\nCommands:\n  init          create local state directories and config\n  config        path, show, or emit config\n  ledger        append or list JSONL spans\n  state         tag, untag, get, list, worktree, delete-slot\n  board-cache   put or get tracker board snapshots\n  capsule       put, get, or prompt compact task context\n  prompt        render kickoff prompts\n  backend       plan or dry-run backend discovery/reconciliation\n  worktree      plan or open backend worktrees/workspaces\n  workspace     register or spawn backend workspaces\n  spawn         plan or run worktree plus primary/critic agents\n  agent         deliver prompts through backend adapters\n  kickoff       plan or deliver queued prompts to agent panes\n  verify        verify prompt or slot execution evidence\n  slot          list, inspect, classify, or mutate managed slots\n  cleanup       cleanup-oriented slot wrappers\n  revamp        revamp-oriented stale slot wrappers\n  status        backend-aware slot status reports\n  cap           check configured work/review caps\n  statusline    print compact pool status\n  work-hours    evaluate configured work-hour guard\n  doctor        debug paths, config, backend, and env\n  selftest      validate local config/state readability\n",
    );
    maeh().args(["state", "--help"]).assert().success().stdout(
        "Manage local slot state\nUsage: maeh state <tag|untag|get|list|worktree|delete-slot>\n",
    );
}

#[test]
fn usage_errors_are_exact() {
    maeh()
        .arg("wat")
        .assert()
        .failure()
        .stderr("maeh error: usage: unknown command wat\n");
    maeh()
        .arg("config")
        .assert()
        .failure()
        .stderr("maeh error: usage: missing config command\n");
    maeh()
        .args(["config", "wat"])
        .assert()
        .failure()
        .stderr("maeh error: usage: unknown config command wat\n");
    maeh()
        .args(["ledger", "wat"])
        .assert()
        .failure()
        .stderr("maeh error: usage: unknown ledger command wat\n");
    maeh()
        .args(["ledger", "append", "--loop"])
        .assert()
        .failure()
        .stderr("maeh error: usage: --loop needs a value\n");
    maeh()
        .args(["state", "wat"])
        .assert()
        .failure()
        .stderr("maeh error: usage: unknown state command wat\n");
    maeh()
        .args(["board-cache", "wat"])
        .assert()
        .failure()
        .stderr("maeh error: usage: unknown board-cache command wat\n");
    maeh()
        .args(["capsule", "wat"])
        .assert()
        .failure()
        .stderr("maeh error: usage: unknown capsule command wat\n");
    let temp = TempDir::new().unwrap();
    maeh()
        .arg("--home")
        .arg(temp.path())
        .args(["capsule", "get", "https://missing.example/task"])
        .assert()
        .failure()
        .stderr("maeh error: cache miss: https://missing.example/task\n");
    maeh()
        .args(["prompt", "wat"])
        .assert()
        .failure()
        .stderr("maeh error: usage: unknown prompt command wat\n");
    maeh()
        .args(["backend", "wat"])
        .assert()
        .failure()
        .stderr("maeh error: usage: unknown backend command wat\n");
    maeh()
        .args(["worktree", "wat"])
        .assert()
        .failure()
        .stderr("maeh error: usage: unknown worktree command wat\n");
    maeh()
        .args(["workspace", "wat"])
        .assert()
        .failure()
        .stderr("maeh error: usage: unknown workspace command wat\n");
    maeh()
        .args(["worktree", "plan"])
        .assert()
        .failure()
        .stderr("maeh error: usage: --slot needs a value\n");
    maeh()
        .args(["spawn", "wat"])
        .assert()
        .failure()
        .stderr("maeh error: usage: unknown spawn command wat\n");
    maeh()
        .args(["agent", "wat"])
        .assert()
        .failure()
        .stderr("maeh error: usage: unknown agent command wat\n");
    maeh()
        .args(["kickoff", "wat", "--target", "p"])
        .assert()
        .failure()
        .stderr("maeh error: usage: unknown kickoff command wat\n");
    maeh()
        .args(["verify", "wat"])
        .assert()
        .failure()
        .stderr("maeh error: usage: unknown verify command wat\n");
    maeh()
        .args(["slot", "wat"])
        .assert()
        .failure()
        .stderr("maeh error: usage: unknown slot command wat\n");
    maeh()
        .args(["cleanup", "wat"])
        .assert()
        .failure()
        .stderr("maeh error: usage: unknown cleanup command wat\n");
    maeh()
        .args(["revamp", "wat"])
        .assert()
        .failure()
        .stderr("maeh error: usage: unknown revamp command wat\n");
    maeh()
        .args(["status", "wat"])
        .assert()
        .failure()
        .stderr("maeh error: usage: unknown status command wat\n");
    maeh()
        .args(["cap", "wat"])
        .assert()
        .failure()
        .stderr("maeh error: usage: unknown cap command wat\n");
    maeh()
        .args(["config", "show"])
        .env("MAEH_BACKEND", "wat")
        .assert()
        .failure()
        .stderr("maeh error: backend: invalid backend: wat\n");
    maeh()
        .args(["backend", "discover", "--fixture", "x", "--exec"])
        .assert()
        .failure()
        .stderr("maeh error: usage: --fixture and --exec are mutually exclusive\n");
}

#[test]
fn init_config_show_doctor_and_home_resolution() {
    let temp = TempDir::new().unwrap();
    let home = temp.path().join("state");
    let expected = format!(
        "maeh\n  created: {0}\n  config: {0}/config.toml\n  ledger: {0}/ledger\n",
        home.display()
    );
    maeh()
        .arg("--home")
        .arg(&home)
        .arg("init")
        .assert()
        .success()
        .stdout(expected.clone());
    maeh()
        .arg("--home")
        .arg(&home)
        .arg("init")
        .assert()
        .success()
        .stdout(expected);
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["config", "path"])
        .assert()
        .success()
        .stdout(format!("{}/config.toml\n", home.display()));
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["config", "show"])
        .assert()
        .success()
        .stdout(format!(
            "maeh config\n  home: {0}\n  backend: auto\n  requested backend: auto\n  selected backend: tmux\n  herdr bin: herdr\n  tmux bin: tmux\n  tmux session: maeh\n  include editor: true\n  focus: false\n  primary agent cmd: codex\n  critic agent cmd: codex\n  editor cmd: vi\n  context switch cap: 3\n  review cap: 5\n  board ttl intake: 3600s\n  board ttl revamp: 10800s\n  capsule max chars: 1800\n  work hours: 9-17\n  workdays: 1,2,3,4,5\n",
            home.display()
        ));
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["config", "emit"])
        .assert()
        .success()
        .stdout("MAEH_BACKEND=auto\nMAEH_HERDR_BIN=herdr\nMAEH_TMUX_BIN=tmux\nMAEH_TMUX_SESSION=maeh\nMAEH_INCLUDE_EDITOR=true\nMAEH_FOCUS=false\nMAEH_PRIMARY_AGENT_CMD=codex\nMAEH_CRITIC_AGENT_CMD=codex\nMAEH_EDITOR_CMD=vi\nMAEH_CONTEXT_SWITCH_CAP=3\nMAEH_REVIEW_CAP=5\nMAEH_BOARD_TTL_INTAKE=3600\nMAEH_BOARD_TTL_REVAMP=10800\nMAEH_TASK_CAPSULE_MAX_CHARS=1800\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["config", "show"])
        .env("MAEH_INCLUDE_EDITOR", "false")
        .env("MAEH_FOCUS", "true")
        .env("MAEH_PRIMARY_AGENT_CMD", "agent primary")
        .env("MAEH_CRITIC_AGENT_CMD", "agent critic")
        .env("MAEH_EDITOR_CMD", "ed")
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["config", "show"])
        .env("MAEH_INCLUDE_EDITOR", "maybe")
        .env("MAEH_FOCUS", "off")
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .arg("doctor")
        .env("HERDR_ENV", "1")
        .env("MAEH_DEBUG", "1")
        .assert()
        .success()
        .stdout(format!(
            "maeh doctor\n  home: {0}\n  config: ok\n  ledger: {0}/ledger\n  backend: auto\n  selected backend: herdr\n  herdr: detected\n  maeh debug: on\n",
            home.display()
        ));
    maeh()
        .arg("--home")
        .arg(&home)
        .arg("doctor")
        .env("HERDR_SOCKET_PATH", "/tmp/herdr.sock")
        .assert()
        .success()
        .stdout(format!(
            "maeh doctor\n  home: {0}\n  config: ok\n  ledger: {0}/ledger\n  backend: auto\n  selected backend: herdr\n  herdr: detected\n  maeh debug: off\n",
            home.display()
        ));
    let env_home = temp.path().join("env-home");
    maeh()
        .env("MAEH_HOME", &env_home)
        .args(["config", "path"])
        .assert()
        .success()
        .stdout(format!("{}/config.toml\n", env_home.display()));
    let fake_home = temp.path().join("unix-home");
    maeh()
        .env("HOME", &fake_home)
        .args(["config", "path"])
        .assert()
        .success()
        .stdout(format!("{}/.maeh/config.toml\n", fake_home.display()));
    maeh()
        .env_remove("HOME")
        .args(["config", "path"])
        .assert()
        .success()
        .stdout(".maeh/config.toml\n");
    maeh()
        .arg("--home")
        .arg(temp.path().join("missing"))
        .arg("doctor")
        .assert()
        .success()
        .stdout(format!(
            "maeh doctor\n  home: {0}/missing\n  config: missing\n  ledger: {0}/missing/ledger\n  backend: auto\n  selected backend: tmux\n  herdr: not-detected\n  maeh debug: off\n",
            temp.path().display()
        ));
}

#[test]
fn backend_dry_run_reconciles_tmux_and_herdr_fixtures() {
    let temp = TempDir::new().unwrap();
    let home = temp.path().join("state");
    init_home(&home);
    let plan_argv = "[\"tmux\",\"list-windows\",\"-t\",\"maeh\",\"-a\",\"-F\",\"#{session_name}:#{window_index}\\u001f#{window_activity}\\u001f#{window_name}\\u001f#{@hmph_task}\\u001f#{@hmph_status}\\u001f#{@hmph_snooze_until}\\u001f#{pane_current_path}\"]";
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["backend", "plan"])
        .assert()
        .success()
        .stdout(format!("maeh backend plan\n  requested backend: auto\n  selected backend: tmux\n  herdr bin: herdr\n  tmux bin: tmux\n  tmux session: maeh\nread\tdiscover\ttmux\tread backend state through adapter; no mutations\n  argv: {plan_argv}\n"));
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["backend", "discover"])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["backend", "reconcile"])
        .assert()
        .success();
    let exec_home = temp.path().join("exec-state");
    init_home(&exec_home);
    fs::write(
        exec_home.join("config.toml"),
        "backend = 'tmux'\ntmux_bin = '/bin/echo'\n",
    )
    .unwrap();
    maeh()
        .arg("--home")
        .arg(&exec_home)
        .args(["backend", "discover", "--exec"])
        .assert()
        .success();
    let fail_home = temp.path().join("fail-state");
    init_home(&fail_home);
    fs::write(
        fail_home.join("config.toml"),
        "backend = 'tmux'\ntmux_bin = '/usr/bin/false'\n",
    )
    .unwrap();
    maeh()
        .arg("--home")
        .arg(&fail_home)
        .args(["backend", "discover", "--exec"])
        .assert()
        .failure()
        .stderr("maeh error: backend: backend command failed: /usr/bin/false exited 1\n");
    let tmux_fixture = temp.path().join("tmux.fixture");
    fs::write(
        &tmux_fixture,
        "orch:1\u{1f}90\u{1f}task-a\u{1f}https://task\u{1f}active\u{1f}0\u{1f}/tmp/wt\n",
    )
    .unwrap();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["backend", "discover", "--fixture"])
        .arg(&tmux_fixture)
        .env("MAEH_EPOCH", "100")
        .assert()
        .success()
        .stdout("maeh backend discover\n  requested backend: auto\n  selected backend: tmux\n  herdr bin: herdr\n  tmux bin: tmux\n  tmux session: maeh\norch:1\thttps://task\tactive\t0\t10\ttask-a\t/tmp/wt\t\t\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "tag", "orch:1", "task_url", "https://task"])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "tag", "orch:2", "task_url", "https://missing"])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["backend", "reconcile", "--fixture"])
        .arg(&tmux_fixture)
        .env("MAEH_EPOCH", "100")
        .assert()
        .success()
        .stdout("maeh backend reconcile\n  requested backend: auto\n  selected backend: tmux\n  herdr bin: herdr\n  tmux bin: tmux\n  tmux session: maeh\nread\tok\torch:1\thttps://task status=active age=10s\nmutate\tmissing-live-slot\torch:2\tlocal state tracks https://missing; dry-run action is delete local slot or respawn explicitly\n");

    let herdr_home = temp.path().join("herdr-state");
    init_home(&herdr_home);
    fs::write(herdr_home.join("config.toml"), "backend = 'herdr'\n").unwrap();
    maeh()
        .arg("--home")
        .arg(&herdr_home)
        .args(["backend", "plan"])
        .assert()
        .success()
        .stdout("maeh backend plan\n  requested backend: herdr\n  selected backend: herdr\n  herdr bin: herdr\n  tmux bin: tmux\n  tmux session: maeh\nread\tdiscover\therdr\tread backend state through adapter; no mutations\n  argv: [\"herdr\",\"api\",\"snapshot\"]\n");
    maeh()
        .arg("--home")
        .arg(&herdr_home)
        .args(["state", "tag", "w1", "task_url", "https://herdr-task"])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&herdr_home)
        .args(["state", "tag", "w1", "status", "active"])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&herdr_home)
        .args(["state", "tag", "w1", "last_activity_epoch", "95"])
        .assert()
        .success();
    let herdr_fixture = temp.path().join("herdr.fixture.json");
    fs::write(
        &herdr_fixture,
        r#"{"result":{"snapshot":{"workspaces":[{"workspace_id":"w1","label":"slot","worktree":{"checkout_path":"/tmp/hwt"}}],"panes":[{"workspace_id":"w1","pane_id":"w1:p2","agent":"primary"},{"workspace_id":"w1","pane_id":"w1:p3","agent":"critic"}]}}}"#,
    )
    .unwrap();
    maeh()
        .arg("--home")
        .arg(&herdr_home)
        .args(["backend", "discover", "--fixture"])
        .arg(&herdr_fixture)
        .env("MAEH_EPOCH", "100")
        .assert()
        .success()
        .stdout("maeh backend discover\n  requested backend: herdr\n  selected backend: herdr\n  herdr bin: herdr\n  tmux bin: tmux\n  tmux session: maeh\nw1\thttps://herdr-task\tactive\t0\t5\tslot\t/tmp/hwt\tw1:p2\tw1:p3\n");
}

#[test]
fn live_orchestration_cli_plans_runs_delivers_and_verifies() {
    let temp = TempDir::new().unwrap();
    let home = temp.path().join("state");
    init_home(&home);
    let herdr = temp.path().join("fake-herdr");
    let herdr_log = temp.path().join("herdr.log");
    let worktree = temp.path().join("wt");
    fs::write(
        &herdr,
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> \"$MAEH_FAKE_LOG\"\nif [ \"$1\" = worktree ]; then printf '{\"result\":{\"workspace_id\":\"w7\",\"path\":\"%s\"}}\\n' \"$MAEH_FAKE_WORKTREE\"; exit 0; fi\nif [ \"$1\" = agent ] && [ \"$2\" = start ]; then case \"$3\" in primary) pane='w7:p2';; critic) pane='w7:p3';; *) pane='w7:p1';; esac; printf '{\"result\":{\"pane_id\":\"%s\"}}\\n' \"$pane\"; exit 0; fi\nif [ \"$1\" = agent ] && [ \"$2\" = read ]; then printf '%s\\n' '{\"result\":{\"read\":{\"text\":\"ready\\n› \"}}}'; exit 0; fi\nprintf '{}\\n'\n",
    )
    .unwrap();
    make_executable(&herdr);
    fs::write(
        home.join("config.toml"),
        format!(
            "backend = 'herdr'\nherdr_bin = '{}'\ninclude_editor = false\nprimary_agent_cmd = 'codex primary'\ncritic_agent_cmd = 'codex critic'\n",
            herdr.display()
        ),
    )
    .unwrap();

    maeh()
        .arg("--home")
        .arg(&home)
        .args([
            "worktree",
            "plan",
            "--slot",
            "w7",
            "--repo",
            "/repo",
            "--branch",
            "ha",
            "--base",
            "main",
            "--path",
        ])
        .arg(&worktree)
        .args(["--label", "live", "--create", "--no-editor", "--no-focus"])
        .assert()
        .success()
        .stdout(format!(
            "maeh worktree plan\n  requested backend: herdr\n  selected backend: herdr\n  herdr bin: {0}\n  tmux bin: tmux\n  tmux session: maeh\nmutate\tworktree-create\tw7\tcreate Herdr worktree/workspace\n  argv: [\"{0}\",\"worktree\",\"create\",\"--cwd\",\"/repo\",\"--branch\",\"ha\",\"--base\",\"main\",\"--path\",\"{1}\",\"--label\",\"live\",\"--no-focus\",\"--json\"]\n",
            herdr.display(),
            worktree.display()
        ));

    maeh()
        .arg("--home")
        .arg(&home)
        .args([
            "worktree", "open", "--slot", "w7", "--repo", "/repo", "--branch", "ha",
            "--base", "main", "--path",
        ])
        .arg(&worktree)
        .args(["--label", "live", "--create", "--no-editor"])
        .env("MAEH_FAKE_LOG", &herdr_log)
        .env("MAEH_FAKE_WORKTREE", &worktree)
        .assert()
        .success()
        .stdout(format!(
            "maeh worktree open\n  requested backend: herdr\n  selected backend: herdr\n  herdr bin: {0}\n  tmux bin: tmux\n  tmux session: maeh\nworktree opened\n  slot: w7\n  workspace: w7\n  path: {1}\n",
            herdr.display(),
            worktree.display()
        ));
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "get", "w7", "workspace_id"])
        .assert()
        .success()
        .stdout("w7\n");

    maeh()
        .arg("--home")
        .arg(&home)
        .args([
            "spawn", "plan", "--slot", "w7", "--repo", "/repo", "--branch", "ha",
            "--base", "main", "--path",
        ])
        .arg(&worktree)
        .args(["--label", "live", "--create", "--task-url", "https://task", "--no-editor"])
        .assert()
        .success()
        .stdout(format!(
            "maeh spawn plan\n  requested backend: herdr\n  selected backend: herdr\n  herdr bin: {0}\n  tmux bin: tmux\n  tmux session: maeh\nmutate\tworktree-create\tw7\tcreate Herdr worktree/workspace\n  argv: [\"{0}\",\"worktree\",\"create\",\"--cwd\",\"/repo\",\"--branch\",\"ha\",\"--base\",\"main\",\"--path\",\"{1}\",\"--label\",\"live\",\"--no-focus\",\"--json\"]\nmutate\tprimary-agent\tw7\tstart primary agent\n  argv: [\"{0}\",\"agent\",\"start\",\"primary\",\"--cwd\",\"{1}\",\"--workspace\",\"$workspace\",\"--split\",\"right\",\"--no-focus\",\"--\",\"codex\",\"primary\"]\nmutate\tcritic-agent\tw7\tstart critic agent\n  argv: [\"{0}\",\"agent\",\"start\",\"critic\",\"--cwd\",\"{1}\",\"--workspace\",\"$workspace\",\"--split\",\"down\",\"--no-focus\",\"--\",\"codex\",\"critic\"]\n",
            herdr.display(),
            worktree.display()
        ));

    maeh()
        .arg("--home")
        .arg(&home)
        .args([
            "spawn", "run", "--slot", "w7", "--repo", "/repo", "--branch", "ha",
            "--base", "main", "--path",
        ])
        .arg(&worktree)
        .args(["--label", "live", "--create", "--task-url", "https://task", "--no-editor"])
        .env("MAEH_FAKE_LOG", &herdr_log)
        .env("MAEH_FAKE_WORKTREE", &worktree)
        .assert()
        .success()
        .stdout(format!(
            "maeh spawn run\n  requested backend: herdr\n  selected backend: herdr\n  herdr bin: {0}\n  tmux bin: tmux\n  tmux session: maeh\nworktree opened\n  slot: w7\n  workspace: w7\n  path: {1}\n  primary pane: w7:p2\n  critic pane: w7:p3\n",
            herdr.display(),
            worktree.display()
        ));
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "get", "w7", "primary_pane"])
        .assert()
        .success()
        .stdout("w7:p2\n");

    maeh()
        .arg("--home")
        .arg(&home)
        .args([
            "spawn", "run", "--slot", "w8", "--repo", "/repo", "--branch", "ha2", "--base", "main",
            "--path",
        ])
        .arg(&worktree)
        .args([
            "--label",
            "live-editor",
            "--create",
            "--task-url",
            "https://task2",
            "--with-editor",
            "--focus",
            "--editor-cmd",
            "vi",
        ])
        .env("MAEH_FAKE_LOG", &herdr_log)
        .env("MAEH_FAKE_WORKTREE", &worktree)
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "get", "w8", "editor_pane"])
        .assert()
        .success()
        .stdout("w7:p1\n");

    maeh()
        .arg("--home")
        .arg(&home)
        .args(["kickoff", "plan", "--target", "w7:p2", "--pane-text", "ready\n› ", "--prompt", "Do it"])
        .assert()
        .success()
        .stdout(format!(
            "maeh kickoff plan\n  requested backend: herdr\n  selected backend: herdr\n  herdr bin: {0}\n  tmux bin: tmux\n  tmux session: maeh\nmutate\tsend-text\tw7:p2\tsend 5 chars plus explicit Enter\n  argv: [\"{0}\",\"agent\",\"send\",\"w7:p2\",\"Do it\"]\nmutate\tsubmit-enter\tw7:p2\tsend 5 chars plus explicit Enter\n  argv: [\"{0}\",\"pane\",\"send-keys\",\"w7:p2\",\"Enter\"]\n",
            herdr.display()
        ));
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["kickoff", "run", "--target", "w7:p2", "--prompt", "Do it"])
        .env("MAEH_FAKE_LOG", &herdr_log)
        .env("MAEH_FAKE_WORKTREE", &worktree)
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args([
            "kickoff",
            "run",
            "--target",
            "w7:p2",
            "--pane-text",
            "ready\n› ",
            "--prompt",
            "Do again",
        ])
        .env("MAEH_FAKE_LOG", &herdr_log)
        .env("MAEH_FAKE_WORKTREE", &worktree)
        .assert()
        .success();
    let prompt_file = temp.path().join("prompt.txt");
    let pane_file = temp.path().join("pane.txt");
    fs::write(&prompt_file, "Do file").unwrap();
    fs::write(&pane_file, "ready\n› ").unwrap();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["kickoff", "plan", "--target", "w7:p2", "--prompt-file"])
        .arg(&prompt_file)
        .arg("--pane-file")
        .arg(&pane_file)
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args([
            "agent",
            "deliver",
            "w7:p2",
            "Do positional",
            "--pane-text",
            "ready\n› ",
            "--exec",
        ])
        .env("MAEH_FAKE_LOG", &herdr_log)
        .env("MAEH_FAKE_WORKTREE", &worktree)
        .assert()
        .success();
    let log = fs::read_to_string(&herdr_log).unwrap();
    assert!(log.contains("agent send w7:p2 Do it"));
    assert!(log.contains("agent send w7:p2 Do again"));
    assert!(log.contains("agent send w7:p2 Do positional"));
    assert!(log.contains("pane send-keys w7:p2 Enter"));

    maeh()
        .args([
            "verify",
            "prompt",
            "--before",
            "› Do it",
            "--after",
            "Working",
            "--prompt",
            "Do it",
        ])
        .assert()
        .success()
        .stdout("maeh verify prompt\n  changed: true\n  submitted: true\n  prompt head: Do it\n");
    let before_file = temp.path().join("before.txt");
    let after_file = temp.path().join("after.txt");
    fs::write(&before_file, "› Do file").unwrap();
    fs::write(&after_file, "Working file").unwrap();
    maeh()
        .args(["verify", "prompt", "--before-file"])
        .arg(&before_file)
        .arg("--after-file")
        .arg(&after_file)
        .arg("--prompt-file")
        .arg(&prompt_file)
        .assert()
        .success();

    let fail_herdr = temp.path().join("fail-herdr");
    fs::write(&fail_herdr, "#!/bin/sh\nexit 7\n").unwrap();
    make_executable(&fail_herdr);
    fs::write(
        home.join("config.toml"),
        format!(
            "backend = 'herdr'\nherdr_bin = '{}'\n",
            fail_herdr.display()
        ),
    )
    .unwrap();
    maeh()
        .arg("--home")
        .arg(&home)
        .args([
            "kickoff",
            "run",
            "--target",
            "w7:p2",
            "--pane-text",
            "ready\n› ",
            "--prompt",
            "Do fail",
        ])
        .assert()
        .failure()
        .stderr(format!(
            "maeh error: backend: backend command failed: {} exited 7\n",
            fail_herdr.display()
        ));
}

#[test]
fn tmux_spawn_run_persists_real_panes_and_delivery_plans_enter() {
    let temp = TempDir::new().unwrap();
    let home = temp.path().join("state");
    init_home(&home);
    let tmux = temp.path().join("fake-tmux");
    fs::write(
        &tmux,
        "#!/bin/sh\nif [ \"$1\" = new-window ]; then printf '@7\\t%%1\\n'; exit 0; fi\nif [ \"$1\" = split-window ]; then printf '%%3\\n'; exit 0; fi\nprintf '{}\\n'\n",
    )
    .unwrap();
    make_executable(&tmux);
    fs::write(
        home.join("config.toml"),
        format!(
            "backend = 'tmux'\ntmux_bin = '{}'\ntmux_session = 'sess'\ninclude_editor = false\n",
            tmux.display()
        ),
    )
    .unwrap();
    let worktree = temp.path().join("wt");
    maeh()
        .arg("--home")
        .arg(&home)
        .args([
            "spawn", "run", "--slot", "t7", "--repo", "/repo", "--branch", "ha",
            "--path",
        ])
        .arg(&worktree)
        .args(["--label", "tmux-live", "--task-url", "https://task", "--no-editor"])
        .assert()
        .success()
        .stdout(format!(
            "maeh spawn run\n  requested backend: tmux\n  selected backend: tmux\n  herdr bin: herdr\n  tmux bin: {0}\n  tmux session: sess\nworktree opened\n  slot: t7\n  workspace: @7\n  window: @7\n  path: {1}\n  primary pane: %1\n  critic pane: %3\n",
            tmux.display(),
            worktree.display()
        ));
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "get", "t7", "critic_pane"])
        .assert()
        .success()
        .stdout("%3\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["kickoff", "plan", "--target", "%1", "--pane-text", "ready\n> ", "--prompt", "Do it"])
        .assert()
        .success()
        .stdout(format!(
            "maeh kickoff plan\n  requested backend: tmux\n  selected backend: tmux\n  herdr bin: herdr\n  tmux bin: {0}\n  tmux session: sess\nmutate\tsend-text\t%1\tsend 5 chars plus explicit Enter\n  argv: [\"{0}\",\"send-keys\",\"-t\",\"%1\",\"-l\",\"Do it\"]\nmutate\tsubmit-enter\t%1\tsend 5 chars plus explicit Enter\n  argv: [\"{0}\",\"send-keys\",\"-t\",\"%1\",\"Enter\"]\n",
            tmux.display()
        ));
}

#[test]
fn kickoff_cli_handles_blockers_and_noops_without_pasting_task_prompt() {
    let temp = TempDir::new().unwrap();
    let home = temp.path().join("state");
    init_home(&home);
    fs::write(home.join("config.toml"), "backend = 'herdr'\n").unwrap();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["kickoff", "plan", "--target", "p", "--pane-text", "Do you trust this folder?", "--prompt", "Do it"])
        .assert()
        .success()
        .stdout("maeh kickoff plan\n  requested backend: herdr\n  selected backend: herdr\n  herdr bin: herdr\n  tmux bin: tmux\n  tmux session: maeh\nmutate\tsend-text\tp\tanswer trust blocker with 1 plus explicit Enter\n  argv: [\"herdr\",\"agent\",\"send\",\"p\",\"1\"]\nmutate\tsubmit-enter\tp\tanswer trust blocker with 1 plus explicit Enter\n  argv: [\"herdr\",\"pane\",\"send-keys\",\"p\",\"Enter\"]\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["kickoff", "plan", "--target", "p", "--pane-text", "Update available. Install?", "--prompt", "Do it"])
        .assert()
        .success()
        .stdout("maeh kickoff plan\n  requested backend: herdr\n  selected backend: herdr\n  herdr bin: herdr\n  tmux bin: tmux\n  tmux session: maeh\nmutate\tsend-text\tp\tanswer update blocker with n plus explicit Enter\n  argv: [\"herdr\",\"agent\",\"send\",\"p\",\"n\"]\nmutate\tsubmit-enter\tp\tanswer update blocker with n plus explicit Enter\n  argv: [\"herdr\",\"pane\",\"send-keys\",\"p\",\"Enter\"]\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["kickoff", "plan", "--target", "p", "--pane-text", "Press Enter to continue", "--prompt", "Do it"])
        .assert()
        .success()
        .stdout("maeh kickoff plan\n  requested backend: herdr\n  selected backend: herdr\n  herdr bin: herdr\n  tmux bin: tmux\n  tmux session: maeh\nmutate\tsubmit-enter\tp\tanswer continue blocker with Enter plus explicit Enter\n  argv: [\"herdr\",\"pane\",\"send-keys\",\"p\",\"Enter\"]\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["kickoff", "plan", "--target", "p", "--pane-text", "working", "--prompt", "Do it"])
        .assert()
        .success()
        .stdout("maeh kickoff plan\n  requested backend: herdr\n  selected backend: herdr\n  herdr bin: herdr\n  tmux bin: tmux\n  tmux session: maeh\nread\tnoop\tp\tpane busy or unknown\n");
}

#[test]
fn slot_lifecycle_contract_matches_skills_calls() {
    let temp = TempDir::new().unwrap();
    let home = temp.path().join("state");
    init_home(&home);
    maeh()
        .arg("--home")
        .arg(&home)
        .args([
            "workspace",
            "register",
            "--slot",
            "s1",
            "--workspace",
            "w1",
            "--worktree",
            "/tmp/wt",
            "--repo",
            "/repo",
            "--task-url",
            "https://task",
            "--primary-pane",
            "p1",
            "--critic-pane",
            "p2",
            "--backend",
            "tmux",
            "--status",
            "active",
        ])
        .assert()
        .success()
        .stdout("workspace registered\n  slot: s1\n  workspace: w1\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args([
            "workspace",
            "register",
            "--slot",
            "s2",
            "--workspace",
            "w2",
            "--worktree",
            "/tmp/wt2",
            "--repo",
            "/repo",
            "--primary-pane",
            "p3",
            "--critic-pane",
            "p4",
            "--backend",
            "tmux",
            "--status",
            "blocked",
        ])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["slot", "count", "--status", "active,blocked"])
        .assert()
        .success()
        .stdout("2\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["slot", "snooze", "s1", "--days", "2", "--status", "blocked"])
        .env("MAEH_EPOCH", "100")
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "get", "s1", "status"])
        .assert()
        .success()
        .stdout("blocked\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "get", "s1", "snooze_until"])
        .assert()
        .success()
        .stdout("172900\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["slot", "remove-worktree", "--slot", "s1", "--plan", "--pull-main"])
        .assert()
        .success()
        .stdout("mutate\tpull-main\ts1\tgit -C /repo pull --ff-only origin main\n  argv: [\"git\",\"-C\",\"/repo\",\"pull\",\"--ff-only\",\"origin\",\"main\"]\nmutate\tremove-worktree\ts1\tgit worktree remove /tmp/wt\n  argv: [\"git\",\"-C\",\"/repo\",\"worktree\",\"remove\",\"/tmp/wt\"]\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["slot", "verify", "--slot", "s1"])
        .assert()
        .success()
        .stdout("slot verified\n  slot: s1\n  status: blocked\n  worktree: /tmp/wt\n  primary pane: p1\n  critic pane: p2\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["backend", "list-task-slots"])
        .assert()
        .success()
        .stdout("maeh backend list-task-slots\n  requested backend: auto\n  selected backend: tmux\n  herdr bin: herdr\n  tmux bin: tmux\n  tmux session: maeh\ns1\thttps://task\tblocked\t172900\t0\t\tp1\tp2\t/tmp/wt\ns2\t\tblocked\t0\t0\t\tp3\tp4\t/tmp/wt2\n");
}

#[test]
fn slot_wrapper_contracts_cover_lifecycle_paths() {
    let temp = TempDir::new().unwrap();
    let home = temp.path().join("state");
    init_home(&home);
    register_slot(&home, "active", "tmux", "active", "/tmp/active", "/repo");
    register_slot(&home, "done", "tmux", "done", "/tmp/done", "/repo");
    register_slot(&home, "review", "tmux", "review", "/tmp/review", "/repo");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "tag", "empty", "status", "active"])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "tag", "stale", "status", "active"])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "tag", "stale", "last_activity_epoch", "1"])
        .assert()
        .success();

    maeh()
        .arg("--home")
        .arg(&home)
        .args(["slot", "inspect", "active"])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["slot", "classify", "active"])
        .assert()
        .success()
        .stdout("active\tactive\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["slot", "list", "--class", "done"])
        .assert()
        .success()
        .stdout("done\thttps://tasks/done\tdone\t0\t0\tdone\t\t/tmp/done\tdone:p\tdone:c\t/repo\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["slot", "list", "--status", "review"])
        .assert()
        .success()
        .stdout("review\thttps://tasks/review\treview\t0\t0\treview\t\t/tmp/review\treview:p\treview:c\t/repo\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["slot", "count", "--class", "done"])
        .assert()
        .success()
        .stdout("1\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["slot", "count", "--status", "missing"])
        .assert()
        .success()
        .stdout("0\n");

    maeh()
        .arg("--home")
        .arg(&home)
        .args(["cleanup", "list"])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["cleanup", "inspect", "--slot", "done"])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["cleanup", "summary"])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["revamp", "list"])
        .env("MAEH_EPOCH", "90000")
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["revamp", "inspect", "active"])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["revamp", "block", "active", "--reason", "waiting"])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["revamp", "snooze", "--slot", "active", "--until", "999"])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["revamp", "resume", "active"])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["revamp", "summary"])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["slot", "block", "active", "--reason", "manual"])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["slot", "resume", "active"])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["slot", "close", "active"])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["slot", "nudge", "active"])
        .env("MAEH_EPOCH", "123")
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "get", "active", "nudge_epoch"])
        .assert()
        .success()
        .stdout("123\n");

    maeh()
        .arg("--home")
        .arg(&home)
        .args(["status", "list"])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["status", "inspect", "active"])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["status", "worktrees"])
        .assert()
        .success()
        .stdout("active\t\t/repo\t\tunknown\t/tmp/active\ndone\t\t/repo\t\tunknown\t/tmp/done\nreview\t\t/repo\t\tunknown\t/tmp/review\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["backend", "list-worktrees"])
        .assert()
        .success()
        .stdout("maeh backend list-worktrees\n  requested backend: auto\n  selected backend: tmux\n  herdr bin: herdr\n  tmux bin: tmux\n  tmux session: maeh\nactive\t\t/repo\t\tunknown\t/tmp/active\ndone\t\t/repo\t\tunknown\t/tmp/done\nreview\t\t/repo\t\tunknown\t/tmp/review\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["cap", "check"])
        .assert()
        .success();
}

#[test]
fn slot_exec_contract_uses_backend_and_git_fakes() {
    let temp = TempDir::new().unwrap();
    let home = temp.path().join("state");
    init_home(&home);
    let bin = temp.path().join("bin");
    fs::create_dir_all(&bin).unwrap();
    let tmux = bin.join("tmux");
    let herdr = bin.join("herdr");
    let git = bin.join("git");
    let log = temp.path().join("exec.log");
    for path in [&tmux, &herdr, &git] {
        fs::write(
            path,
            "#!/bin/sh\nprintf '%s %s\n' \"$(basename \"$0\")\" \"$*\" >> \"$MAEH_FAKE_LOG\"\nexit 0\n",
        )
        .unwrap();
        make_executable(path);
    }
    fs::write(
        home.join("config.toml"),
        format!(
            "backend = 'tmux'\ntmux_bin = '{}'\nherdr_bin = '{}'\n",
            tmux.display(),
            herdr.display()
        ),
    )
    .unwrap();
    register_slot(&home, "tmux-close", "tmux", "active", "/tmp/tmux", "/repo");
    register_slot(
        &home,
        "herdr-close",
        "herdr",
        "active",
        "/tmp/herdr",
        "/repo",
    );
    register_slot(&home, "git-remove", "tmux", "active", "/tmp/git", "/repo");

    maeh()
        .arg("--home")
        .arg(&home)
        .args(["slot", "close", "tmux-close", "--exec"])
        .env("MAEH_FAKE_LOG", &log)
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["slot", "close", "herdr-close", "--exec"])
        .env("MAEH_FAKE_LOG", &log)
        .assert()
        .success();
    let path = format!("{}:{}", bin.display(), std::env::var("PATH").unwrap());
    maeh()
        .arg("--home")
        .arg(&home)
        .args([
            "slot",
            "worktree-remove",
            "git-remove",
            "--exec",
            "--pull-main",
        ])
        .env("MAEH_FAKE_LOG", &log)
        .env("PATH", path)
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "get", "tmux-close", "status"])
        .assert()
        .failure()
        .stderr("maeh error: cache miss: tmux-close:status\n");
    let log = fs::read_to_string(&log).unwrap();
    assert!(log.contains("tmux kill-window -t workspace-tmux-close"));
    assert!(log.contains("herdr workspace close workspace-herdr-close"));
    assert!(log.contains("git -C /repo pull --ff-only origin main"));
    assert!(log.contains("git -C /repo worktree remove /tmp/git"));
}

#[test]
fn slot_spawn_and_agent_contract_cover_aliases_and_exec() {
    let temp = TempDir::new().unwrap();
    let home = temp.path().join("state");
    init_home(&home);
    let herdr = temp.path().join("fake-herdr");
    let herdr_log = temp.path().join("herdr.log");
    let worktree = temp.path().join("spawned-wt");
    fs::write(
        &herdr,
        "#!/bin/sh\nprintf '%s\n' \"$*\" >> \"$MAEH_FAKE_LOG\"\nif [ \"$1\" = worktree ]; then printf '{\"result\":{\"workspace_id\":\"wspawn\",\"path\":\"%s\"}}\\n' \"$MAEH_FAKE_WORKTREE\"; exit 0; fi\nif [ \"$1\" = agent ] && [ \"$2\" = start ]; then case \"$3\" in primary) pane='wspawn:p';; critic) pane='wspawn:c';; *) pane='wspawn:e';; esac; printf '{\"result\":{\"pane_id\":\"%s\"}}\\n' \"$pane\"; exit 0; fi\nif [ \"$1\" = agent ] && [ \"$2\" = read ]; then printf '%s\\n' '{\"result\":{\"read\":{\"text\":\"ready\\n› \"}}}'; exit 0; fi\nprintf '{}\\n'\n",
    )
    .unwrap();
    make_executable(&herdr);
    fs::write(
        home.join("config.toml"),
        format!(
            "backend = 'herdr'\nherdr_bin = '{}'\ninclude_editor = true\nprimary_agent_cmd = 'codex primary'\ncritic_agent_cmd = 'codex critic'\neditor_cmd = 'vi edit'\n",
            herdr.display()
        ),
    )
    .unwrap();

    maeh()
        .arg("--home")
        .arg(&home)
        .args([
            "workspace",
            "spawn",
            "--backend",
            "tmux",
            "--label",
            "from-label",
            "--repo-root",
            "/repo",
            "--branch",
            "feat/from-label",
            "--worktree",
            "/tmp/from-label",
            "--task-url",
            "https://tasks/from-label",
            "--editor",
            "false",
            "--open-existing",
        ])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args([
            "slot",
            "spawn",
            "--backend",
            "herdr",
            "--slot",
            "spawned",
            "--repo",
            "/repo",
            "--branch",
            "feat/spawned",
            "--path",
            "/tmp/spawned",
            "--task-url",
            "https://tasks/spawned",
            "--editor",
            "true",
            "--exec",
        ])
        .env("MAEH_FAKE_LOG", &herdr_log)
        .env("MAEH_FAKE_WORKTREE", &worktree)
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "get", "spawned", "label"])
        .assert()
        .success()
        .stdout("spawned\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "get", "spawned", "branch"])
        .assert()
        .success()
        .stdout("feat/spawned\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args([
            "agent", "deliver", "--slot", "spawned", "--role", "bad", "--prompt", "noop",
        ])
        .assert()
        .failure()
        .stderr("maeh error: usage: unknown role bad\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["agent", "deliver", "--slot", "missing", "--prompt", "noop"])
        .assert()
        .failure()
        .stderr("maeh error: cache miss: missing\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args([
            "agent",
            "deliver",
            "--slot",
            "spawned",
            "--role",
            "critic",
            "--prompt",
            "Review",
            "--pane-text",
            "ready\n› ",
        ])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args([
            "kickoff",
            "run",
            "--slot",
            "spawned",
            "--prompt",
            "Go",
            "--pane-text",
            "ready\n› ",
        ])
        .env("MAEH_FAKE_LOG", &herdr_log)
        .env("MAEH_FAKE_WORKTREE", &worktree)
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args([
            "slot", "nudge", "spawned", "--role", "critic", "--prompt", "Again",
        ])
        .env("MAEH_FAKE_LOG", &herdr_log)
        .env("MAEH_FAKE_WORKTREE", &worktree)
        .assert()
        .success();
    maeh()
        .args(["agent", "deliver", "--prompt", "missing target"])
        .assert()
        .failure()
        .stderr("maeh error: usage: --target or --slot needs a value\n");

    let log = fs::read_to_string(&herdr_log).unwrap();
    assert!(log.contains("worktree create --cwd /repo --branch feat/spawned --base HEAD --path /tmp/spawned --label spawned --no-focus --json"));
    assert!(log.contains("agent start primary --cwd "));
    assert!(log.contains(" --workspace wspawn --split right --no-focus -- codex primary"));
    assert!(log.contains(" --workspace wspawn --split down --no-focus -- codex critic"));
    assert!(log.contains(" --workspace wspawn --split right --no-focus -- vi edit"));
    assert!(log.contains("agent send wspawn:c Go"));
    assert!(log.contains("agent send wspawn:c Again"));
}

#[test]
fn backend_list_task_slots_fixture_uses_backend_discovery() {
    let temp = TempDir::new().unwrap();
    let home = temp.path().join("state");
    init_home(&home);
    let fixture = temp.path().join("tmux.fixture");
    fs::write(
        &fixture,
        "orch:1\u{1f}90\u{1f}task-a\u{1f}https://task\u{1f}active\u{1f}0\u{1f}/tmp/wt\n",
    )
    .unwrap();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["backend", "list-task-slots", "--fixture"])
        .arg(&fixture)
        .env("MAEH_EPOCH", "100")
        .assert()
        .success()
        .stdout("maeh backend list-task-slots\n  requested backend: auto\n  selected backend: tmux\n  herdr bin: herdr\n  tmux bin: tmux\n  tmux session: maeh\norch:1\thttps://task\tactive\t0\t10\ttask-a\t\t\t/tmp/wt\n");
}

#[test]
fn slot_error_paths_cover_contract_failures() {
    let temp = TempDir::new().unwrap();
    let home = temp.path().join("state");
    init_home(&home);
    maeh()
        .arg("--home")
        .arg(&home)
        .args([
            "slot",
            "spawn",
            "--backend",
            "wat",
            "--slot",
            "bad",
            "--repo",
            "/repo",
            "--path",
            "/tmp/bad",
            "--task-url",
            "https://tasks/bad",
        ])
        .assert()
        .failure()
        .stderr("maeh error: backend: invalid backend: wat\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args([
            "workspace",
            "register",
            "--slot",
            "with-editor",
            "--workspace",
            "workspace-with-editor",
            "--worktree",
            "/tmp/editor",
            "--editor-pane",
            "with-editor:e",
        ])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "get", "with-editor", "editor_pane"])
        .assert()
        .success()
        .stdout("with-editor:e\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "tag", "partial", "worktree", "/tmp/partial"])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args([
            "state",
            "tag",
            "primary-only",
            "primary_pane",
            "primary-only:p",
        ])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args([
            "state",
            "tag",
            "critic-only",
            "critic_pane",
            "critic-only:c",
        ])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "tag", "no-status", "worktree", "/tmp/no-status"])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "tag", "repo-only", "repo", "/repo"])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["slot", "verify", "missing"])
        .assert()
        .failure()
        .stderr("maeh error: cache miss: missing\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["slot", "verify", "partial"])
        .assert()
        .failure()
        .stderr("maeh error: cache miss: partial:primary_pane\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["slot", "inspect", "missing"])
        .assert()
        .failure()
        .stderr("maeh error: cache miss: missing\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["slot", "classify", "missing"])
        .assert()
        .failure()
        .stderr("maeh error: cache miss: missing\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["slot", "classify", "no-status"])
        .assert()
        .success()
        .stdout("no-status\tnone\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args([
            "agent",
            "deliver",
            "--slot",
            "primary-only",
            "--role",
            "critic",
            "--prompt",
            "nope",
        ])
        .assert()
        .failure()
        .stderr("maeh error: cache miss: primary-only:critic_pane\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args([
            "agent",
            "deliver",
            "--slot",
            "critic-only",
            "--role",
            "primary",
            "--prompt",
            "nope",
        ])
        .assert()
        .failure()
        .stderr("maeh error: cache miss: critic-only:primary_pane\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["slot", "worktree-remove", "missing"])
        .assert()
        .failure()
        .stderr("maeh error: cache miss: missing\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["slot", "worktree-remove", "partial"])
        .assert()
        .failure()
        .stderr("maeh error: cache miss: partial:repo\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["slot", "worktree-remove", "repo-only"])
        .assert()
        .failure()
        .stderr("maeh error: cache miss: repo-only:worktree\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["slot", "close", "missing"])
        .assert()
        .failure()
        .stderr("maeh error: cache miss: missing\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["slot", "close", "partial"])
        .assert()
        .failure()
        .stderr("maeh error: cache miss: partial:workspace_id\n");
    register_slot(
        &home,
        "failing-close",
        "tmux",
        "active",
        "/tmp/failing",
        "/repo",
    );
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["slot", "close", "failing-close", "--exec"])
        .env("MAEH_TMUX_BIN", "/usr/bin/false")
        .assert()
        .failure()
        .stderr("maeh error: backend: backend command failed: /usr/bin/false exited 1\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["slot", "close", "failing-close"])
        .env("MAEH_BACKEND", "wat")
        .assert()
        .failure()
        .stderr("maeh error: backend: invalid backend: wat\n");
}

#[test]
fn ledger_append_list_and_json_errors_are_line_asserted() {
    let temp = TempDir::new().unwrap();
    let home = temp.path().join("state");
    init_home(&home);
    maeh()
        .arg("--home")
        .arg(&home)
        .args([
            "ledger",
            "append",
            "--loop",
            "daily",
            "--event",
            "run_start",
            "--target",
            "w1",
            "--data",
            "{\"ok\":true}",
        ])
        .env("MAEH_NOW", "2026-07-16T00:00:00Z")
        .assert()
        .success()
        .stdout(format!(
            "ledger appended\n  file: {}/ledger/daily.jsonl\n  event: run_start\n  target: w1\n",
            home.display()
        ));
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["ledger", "list", "--loop", "daily"])
        .assert()
        .success()
        .stdout("2026-07-16T00:00:00Z run_start w1 {\"ok\":true}\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args([
            "ledger", "append", "--loop", "daily", "--event", "bad", "--data", "nope",
        ])
        .assert()
        .failure()
        .stderr("maeh error: json: expected ident at line 1 column 2\n");
}

#[test]
fn state_statusline_work_hours_and_selftest_are_exact() {
    let temp = TempDir::new().unwrap();
    let home = temp.path().join("state");
    init_home(&home);
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "tag", "w1", "task_url", "https://task"])
        .assert()
        .success()
        .stdout("state tagged\n  slot: w1\n  task_url: https://task\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "tag", "w1", "status", "active"])
        .assert()
        .success()
        .stdout("state tagged\n  slot: w1\n  status: active\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "tag", "w1", "worktree", "/tmp/wt"])
        .assert()
        .success()
        .stdout("state tagged\n  slot: w1\n  worktree: /tmp/wt\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "get", "w1", "task_url"])
        .assert()
        .success()
        .stdout("https://task\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "worktree", "w1"])
        .assert()
        .success()
        .stdout("/tmp/wt\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "list"])
        .assert()
        .success()
        .stdout("w1\thttps://task\tactive\t0\t/tmp/wt\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "tag", "w2", "status", "review"])
        .assert()
        .success()
        .stdout("state tagged\n  slot: w2\n  status: review\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "tag", "w3", "status", "done"])
        .assert()
        .success()
        .stdout("state tagged\n  slot: w3\n  status: done\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .arg("statusline")
        .assert()
        .success()
        .stdout("maeh W:1/3 R:1/5\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "untag", "w1", "status"])
        .assert()
        .success()
        .stdout("state untagged\n  slot: w1\n  key: status\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "get", "w1", "status"])
        .assert()
        .failure()
        .stderr("maeh error: cache miss: w1:status\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "untag", "missing", "status"])
        .assert()
        .success()
        .stdout("state untagged\n  slot: missing\n  key: status\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "delete-slot", "w1"])
        .assert()
        .success()
        .stdout("state slot deleted\n  slot: w1\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "delete-slot", "w2"])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "delete-slot", "w3"])
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["state", "list"])
        .assert()
        .success()
        .stdout("");
    maeh()
        .arg("--home")
        .arg(&home)
        .arg("work-hours")
        .env("MAEH_DOW", "2")
        .env("MAEH_HOUR", "10")
        .assert()
        .success()
        .stdout("work-hours\n  day: 2\n  hour: 10\n  active: true\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .arg("work-hours")
        .env("MAEH_DOW", "7")
        .env("MAEH_HOUR", "10")
        .assert()
        .success()
        .stdout("work-hours\n  day: 7\n  hour: 10\n  active: false\n");
    let (bad_dow, bad_hour, bad_active) = local_work_time();
    maeh()
        .arg("--home")
        .arg(&home)
        .arg("work-hours")
        .env("MAEH_DOW", "bad")
        .env("MAEH_HOUR", "bad")
        .assert()
        .success()
        .stdout(format!(
            "work-hours\n  day: {bad_dow}\n  hour: {bad_hour}\n  active: {bad_active}\n"
        ));
    let (dow, hour, active) = local_work_time();
    maeh()
        .arg("--home")
        .arg(&home)
        .arg("work-hours")
        .assert()
        .success()
        .stdout(format!(
            "work-hours\n  day: {dow}\n  hour: {hour}\n  active: {active}\n"
        ));
    maeh()
        .arg("--home")
        .arg(&home)
        .arg("selftest")
        .assert()
        .success()
        .stdout("maeh selftest\n  config: ok\n  state: ok\n");
}

#[test]
fn board_cache_respects_ttl_stale_reads_and_missing_errors() {
    let temp = TempDir::new().unwrap();
    let home = temp.path().join("state");
    init_home(&home);
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["board-cache", "put", "--key", "intake"])
        .env("MAEH_NOW", "2026-07-16T01:00:00Z")
        .env("MAEH_EPOCH", "100")
        .write_stdin("{\"rows\":[1]}")
        .assert()
        .success()
        .stdout(format!(
            "board cache stored\n  key: intake\n  file: {}/board-cache/intake.json\n",
            home.display()
        ));
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["board-cache", "get", "--key", "intake"])
        .env("MAEH_EPOCH", "200")
        .assert()
        .success()
        .stdout("{\"rows\":[1]}\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["board-cache", "get", "--key", "intake"])
        .env("MAEH_EPOCH", "4000")
        .assert()
        .failure()
        .stderr("maeh error: cache miss: intake\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["board-cache", "get", "--key", "intake", "--stale"])
        .env("MAEH_EPOCH", "4000")
        .assert()
        .success()
        .stdout("{\"rows\":[1]}\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["board-cache", "get", "--key", "missing"])
        .assert()
        .failure()
        .stderr("maeh error: cache miss: missing\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["board-cache", "put", "--key", "noenv"])
        .env("MAEH_EPOCH", "bad")
        .write_stdin("{\"rows\":[0]}")
        .assert()
        .success()
        .stdout(format!(
            "board cache stored\n  key: noenv\n  file: {}/board-cache/noenv.json\n",
            home.display()
        ));
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["board-cache", "put", "--key", "runtime"])
        .write_stdin("{\"rows\":[3]}")
        .assert()
        .success()
        .stdout(format!(
            "board cache stored\n  key: runtime\n  file: {}/board-cache/runtime.json\n",
            home.display()
        ));
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["board-cache", "put", "--key", "revamp"])
        .env("MAEH_EPOCH", "100")
        .write_stdin("{\"rows\":[2]}")
        .assert()
        .success();
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["board-cache", "get", "--key", "revamp"])
        .env("MAEH_EPOCH", "10000")
        .assert()
        .success()
        .stdout("{\"rows\":[2]}\n");
}

#[test]
fn capsule_put_get_prompt_and_kickoff_are_exact() {
    let temp = TempDir::new().unwrap();
    let home = temp.path().join("state");
    init_home(&home);
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["capsule", "put", "https://task", "--edited", "v1"])
        .env("MAEH_NOW", "now")
        .env("MAEH_EPOCH", "7")
        .write_stdin("{\"title\":\"Build\",\"next_step\":\"test\"}")
        .assert()
        .success()
        .stdout(format!(
            "capsule stored\n  url: https://task\n  file: {}/task-capsules/1da4a60703deb204.json\n",
            home.display()
        ));
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["capsule", "get", "https://task", "--edited", "v1"])
        .assert()
        .success()
        .stdout("{\"next_step\":\"test\",\"title\":\"Build\"}\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["capsule", "prompt", "https://task"])
        .assert()
        .success()
        .stdout("Task capsule\n```json\n{\"next_step\":\"test\",\"title\":\"Build\"}\n```\n");
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["capsule", "get", "https://task", "--edited", "v2"])
        .assert()
        .failure()
        .stderr("maeh error: cache miss: https://task\n");
    let tiny = temp.path().join("tiny");
    fs::create_dir_all(&tiny).unwrap();
    fs::write(tiny.join("config.toml"), "backend = 'auto'\ncontext_switch_cap = 3\nreview_cap = 5\nboard_ttl_intake_secs = 1\nboard_ttl_revamp_secs = 1\ntask_capsule_max_chars = 2\nwork_start_hour = 9\nwork_end_hour = 17\nworkdays = [1,2,3,4,5]\n").unwrap();
    maeh()
        .arg("--home")
        .arg(&tiny)
        .args(["capsule", "put", "https://task"])
        .write_stdin("{\"abc\":true}")
        .assert()
        .failure()
        .stderr("maeh error: capsule too large: 12 chars > 2 chars\n");
    let capsule_file = temp.path().join("capsule.json");
    fs::write(&capsule_file, "{\"title\":\"Build\"}\n").unwrap();
    maeh()
        .args(["prompt", "kickoff", "--url", "https://task", "--capsule-file"])
        .arg(&capsule_file)
        .assert()
        .success()
        .stdout("Maeh kickoff\n  task: https://task\n  instruction: use the capsule first; fetch tracker context only if stale or insufficient\n  guardrail: plan with the critic before writing code\nTask capsule\n```json\n{\"title\":\"Build\"}\n```\n");
    maeh()
        .args(["prompt", "kickoff", "--url", "https://task"])
        .assert()
        .success()
        .stdout("Maeh kickoff\n  task: https://task\n  instruction: use the capsule first; fetch tracker context only if stale or insufficient\n  guardrail: plan with the critic before writing code\nTask capsule\n```json\n{}\n```\n");
}
