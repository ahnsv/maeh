use std::path::Path;

use chrono::{Datelike, Timelike};
use maeh::backend::BackendEnv;

use crate::commands::state::read_state;
use crate::config::{backend_settings_for_config_env, config_path, ledger_dir, read_config};
use crate::error::Result;
use crate::util::display;

pub(crate) fn doctor(home: &Path) -> Result<()> {
    let config = read_config(home)?;
    let backend_env = BackendEnv::from_env()?;
    let settings = backend_settings_for_config_env(&config, &backend_env)?;
    let config_state = if config_path(home).exists() {
        "ok"
    } else {
        "missing"
    };
    let herdr_state = if backend_env.herdr_session {
        "detected"
    } else {
        "not-detected"
    };
    let debug_state = if std::env::var_os("MAEH_DEBUG").is_some() {
        "on"
    } else {
        "off"
    };
    println!("maeh doctor");
    println!("  home: {}", display(home));
    println!("  config: {config_state}");
    println!("  ledger: {}", display(&ledger_dir(home)));
    println!("  backend: {}", config.backend.kind);
    println!("  selected backend: {}", settings.selected);
    println!("  herdr: {herdr_state}");
    println!("  maeh debug: {debug_state}");
    Ok(())
}

pub(crate) fn statusline(home: &Path) -> Result<()> {
    let config = read_config(home)?;
    let mut work = 0;
    let mut review = 0;
    for entry in read_state(home)?.into_values() {
        match entry.get("status").map(String::as_str) {
            Some("active" | "blocked") => work += 1,
            Some("review") => review += 1,
            _ => {}
        }
    }
    println!(
        "maeh W:{}/{} R:{}/{}",
        work, config.limits.context_switch_cap, review, config.limits.review_cap
    );
    Ok(())
}

pub(crate) fn work_hours(home: &Path) -> Result<()> {
    let config = read_config(home)?;
    let (dow, hour) = current_dow_hour();
    let active = config.work_hours.workdays.contains(&dow)
        && hour >= config.work_hours.start_hour
        && hour < config.work_hours.end_hour;
    println!("work-hours");
    println!("  day: {dow}");
    println!("  hour: {hour}");
    println!("  active: {active}");
    Ok(())
}

fn current_dow_hour() -> (u32, u32) {
    let now = chrono::Local::now();
    let dow = match std::env::var("MAEH_DOW") {
        Ok(value) => value
            .parse()
            .unwrap_or_else(|_| now.weekday().number_from_monday()),
        Err(_) => now.weekday().number_from_monday(),
    };
    let hour = match std::env::var("MAEH_HOUR") {
        Ok(value) => value.parse().unwrap_or_else(|_| now.hour()),
        Err(_) => now.hour(),
    };
    (dow, hour)
}

pub(crate) fn selftest(home: &Path) -> Result<()> {
    let _ = read_config(home)?;
    let _ = read_state(home)?;
    println!("maeh selftest");
    println!("  config: ok");
    println!("  state: ok");
    Ok(())
}
