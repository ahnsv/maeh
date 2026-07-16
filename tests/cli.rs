use std::fs;
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

fn local_work_time() -> (u32, u32, bool) {
    let now = chrono::Local::now();
    let dow = now.weekday().number_from_monday();
    let hour = now.hour();
    (dow, hour, (1..=5).contains(&dow) && (9..17).contains(&hour))
}

#[test]
fn help_output_is_styled_and_lists_core_commands() {
    maeh().arg("--help").assert().success().stdout(
        "Typed orchestration CLI for hmph and Herdr agents\n\nUsage: maeh [--home PATH] <command>\n\nCommands:\n  init          create local state directories and config\n  config        path, show, or emit config\n  ledger        append or list JSONL spans\n  state         tag, untag, get, list, worktree, delete-slot\n  board-cache   put or get tracker board snapshots\n  capsule       put, get, or prompt compact task context\n  prompt        render kickoff prompts\n  backend       plan or dry-run backend discovery/reconciliation\n  statusline    print compact pool status\n  work-hours    evaluate configured work-hour guard\n  doctor        debug paths, config, backend, and env\n  selftest      validate local config/state readability\n",
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
            "maeh config\n  home: {0}\n  backend: auto\n  requested backend: auto\n  selected backend: tmux\n  herdr bin: herdr\n  tmux bin: tmux\n  context switch cap: 3\n  review cap: 5\n  board ttl intake: 3600s\n  board ttl revamp: 10800s\n  capsule max chars: 1800\n  work hours: 9-17\n  workdays: 1,2,3,4,5\n",
            home.display()
        ));
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["config", "emit"])
        .assert()
        .success()
        .stdout("MAEH_BACKEND=auto\nMAEH_HERDR_BIN=herdr\nMAEH_TMUX_BIN=tmux\nMAEH_CONTEXT_SWITCH_CAP=3\nMAEH_REVIEW_CAP=5\nMAEH_BOARD_TTL_INTAKE=3600\nMAEH_BOARD_TTL_REVAMP=10800\nMAEH_TASK_CAPSULE_MAX_CHARS=1800\n");
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
    let plan_argv = "[\"tmux\",\"list-windows\",\"-a\",\"-F\",\"#{session_name}:#{window_index}\\u001f#{window_activity}\\u001f#{window_name}\\u001f#{@hmph_task}\\u001f#{@hmph_status}\\u001f#{@hmph_snooze_until}\\u001f#{pane_current_path}\"]";
    maeh()
        .arg("--home")
        .arg(&home)
        .args(["backend", "plan"])
        .assert()
        .success()
        .stdout(format!("maeh backend plan\n  requested backend: auto\n  selected backend: tmux\n  herdr bin: herdr\n  tmux bin: tmux\nread\tdiscover\ttmux\tread backend state through adapter; no mutations\n  argv: {plan_argv}\n"));
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
        .stdout("maeh backend discover\n  requested backend: auto\n  selected backend: tmux\n  herdr bin: herdr\n  tmux bin: tmux\norch:1\thttps://task\tactive\t0\t10\ttask-a\t/tmp/wt\torch:1.1\torch:1.2\n");
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
        .stdout("maeh backend reconcile\n  requested backend: auto\n  selected backend: tmux\n  herdr bin: herdr\n  tmux bin: tmux\nread\tok\torch:1\thttps://task status=active age=10s\nmutate\tmissing-live-slot\torch:2\tlocal state tracks https://missing; dry-run action is delete local slot or respawn explicitly\n");

    let herdr_home = temp.path().join("herdr-state");
    init_home(&herdr_home);
    fs::write(herdr_home.join("config.toml"), "backend = 'herdr'\n").unwrap();
    maeh()
        .arg("--home")
        .arg(&herdr_home)
        .args(["backend", "plan"])
        .assert()
        .success()
        .stdout("maeh backend plan\n  requested backend: herdr\n  selected backend: herdr\n  herdr bin: herdr\n  tmux bin: tmux\nread\tdiscover\therdr\tread backend state through adapter; no mutations\n  argv: [\"herdr\",\"api\",\"snapshot\"]\n");
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
        .stdout("maeh backend discover\n  requested backend: herdr\n  selected backend: herdr\n  herdr bin: herdr\n  tmux bin: tmux\nw1\thttps://herdr-task\tactive\t0\t5\tslot\t/tmp/hwt\tw1:p2\tw1:p3\n");
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
