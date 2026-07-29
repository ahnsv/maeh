use std::fs;
use std::path::Path;

use crate::config::{config_path, ledger_dir, Config};
use crate::error::Result;
use crate::util::{display, write_file};

pub(crate) fn init(home: &Path) -> Result<()> {
    fs::create_dir_all(ledger_dir(home))?;
    fs::create_dir_all(home.join("board-cache"))?;
    fs::create_dir_all(home.join("task-capsules"))?;
    let path = config_path(home);
    if !path.exists() {
        let config_text = toml::to_string_pretty(&Config::default())?;
        write_file(&path, config_text.as_bytes())?;
    }
    println!("maeh");
    println!("  created: {}", display(home));
    println!("  config: {}", display(&path));
    println!("  ledger: {}", display(&ledger_dir(home)));
    Ok(())
}
