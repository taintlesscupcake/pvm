//! Doctor command — health-check the PVM installation and shell integration

use anyhow::Result;
use console::style;
use pvm_core::Config;
use std::env;
use std::path::PathBuf;
use std::time::SystemTime;

pub fn execute() -> Result<()> {
    let mut report = Report::default();

    println!("{}", style("PVM Doctor").bold());
    println!();

    check_binary(&mut report);
    check_path(&mut report);
    check_legacy_binary(&mut report);
    check_home(&mut report);
    check_config(&mut report);
    check_shell_integration(&mut report);
    check_metadata_freshness(&mut report);

    println!();
    report.summary();
    if report.fail > 0 {
        std::process::exit(1);
    }
    Ok(())
}

#[derive(Default)]
struct Report {
    pass: u32,
    warn: u32,
    fail: u32,
}

impl Report {
    fn ok(&mut self, label: &str, detail: impl AsRef<str>) {
        println!(
            "  {} {}: {}",
            style("✓").green(),
            label,
            style(detail.as_ref()).dim()
        );
        self.pass += 1;
    }

    fn warn(&mut self, label: &str, detail: impl AsRef<str>, hint: Option<&str>) {
        println!(
            "  {} {}: {}",
            style("!").yellow(),
            label,
            detail.as_ref()
        );
        if let Some(h) = hint {
            println!("      {} {}", style("→").dim(), style(h).dim());
        }
        self.warn += 1;
    }

    fn fail(&mut self, label: &str, detail: impl AsRef<str>, hint: Option<&str>) {
        println!(
            "  {} {}: {}",
            style("✗").red(),
            label,
            detail.as_ref()
        );
        if let Some(h) = hint {
            println!("      {} {}", style("→").dim(), style(h).dim());
        }
        self.fail += 1;
    }

    fn summary(&self) {
        let parts = format!(
            "{} ok, {} warning, {} error",
            self.pass, self.warn, self.fail
        );
        if self.fail > 0 {
            println!("{}: {}", style("Result").bold().red(), parts);
        } else if self.warn > 0 {
            println!("{}: {}", style("Result").bold().yellow(), parts);
        } else {
            println!("{}: {}", style("Result").bold().green(), parts);
        }
    }
}

fn check_binary(report: &mut Report) {
    match env::current_exe() {
        Ok(path) => report.ok("Binary", path.display().to_string()),
        Err(e) => report.fail("Binary", format!("cannot resolve current exe: {}", e), None),
    }
}

fn check_path(report: &mut Report) {
    let exe = match env::current_exe() {
        Ok(p) => p,
        Err(_) => return,
    };
    let bin_dir = match exe.parent() {
        Some(p) => p.to_path_buf(),
        None => return,
    };

    let path_env = env::var_os("PATH").unwrap_or_default();
    let on_path = env::split_paths(&path_env).any(|p| p == bin_dir);

    if on_path {
        report.ok("PATH", format!("{} is on PATH", bin_dir.display()));
    } else {
        let hint = format!(
            "add `export PATH=\"{}:$PATH\"` to your shell rc",
            bin_dir.display()
        );
        report.warn(
            "PATH",
            format!("{} is not on PATH", bin_dir.display()),
            Some(&hint),
        );
    }
}

fn check_legacy_binary(report: &mut Report) {
    let home = home_dir();
    let legacy = home.join(".pvm/bin/pvm");
    if !legacy.exists() {
        return;
    }

    let current = env::current_exe().ok();
    let is_current = current.as_deref() == Some(legacy.as_path());

    if is_current {
        report.warn(
            "Legacy binary",
            format!("running from old location {}", legacy.display()),
            Some("reinstall with the one-liner to move binary to ~/.local/bin"),
        );
    } else {
        report.warn(
            "Legacy binary",
            format!("stale binary at {} (remove it)", legacy.display()),
            Some(&format!("rm {}", legacy.display())),
        );
    }
}

fn check_home(report: &mut Report) {
    let config = Config::load().unwrap_or_default();
    if !config.home.exists() {
        report.fail(
            "PVM_HOME",
            format!("{} does not exist", config.home.display()),
            Some("re-run the install script"),
        );
        return;
    }

    report.ok("PVM_HOME", config.home.display().to_string());

    for (label, dir) in [
        ("pythons dir", config.pythons_dir()),
        ("envs dir", config.envs_dir()),
        ("cache dir", config.cache_dir()),
    ] {
        if dir.exists() {
            report.ok(label, dir.display().to_string());
        } else {
            report.warn(label, format!("missing: {}", dir.display()), None);
        }
    }
}

fn check_config(report: &mut Report) {
    let path = match Config::config_path() {
        Ok(p) => p,
        Err(e) => {
            report.fail("Config", format!("cannot resolve path: {}", e), None);
            return;
        }
    };

    if !path.exists() {
        report.warn(
            "Config",
            format!("{} missing — using defaults", path.display()),
            Some("run `pvm config sync` to write defaults"),
        );
        return;
    }

    match Config::load() {
        Ok(_) => report.ok("Config", path.display().to_string()),
        Err(e) => report.fail(
            "Config",
            format!("{} is invalid: {}", path.display(), e),
            Some("fix or delete the file and re-run `pvm config sync`"),
        ),
    }
}

fn check_shell_integration(report: &mut Report) {
    if env::var_os("PVM_SHELL_INTEGRATION").is_some() {
        report.ok("Shell integration", "loaded");
    } else {
        report.warn(
            "Shell integration",
            "not loaded in this shell",
            Some("add `eval \"$(pvm init zsh)\"` (or bash) to your shell rc"),
        );
    }
}

fn check_metadata_freshness(report: &mut Report) {
    let config = Config::load().unwrap_or_default();
    let path = config.home.join("python-metadata.json");
    if !path.exists() {
        report.warn(
            "Python metadata",
            "not yet downloaded",
            Some("run `pvm update` to fetch the version list"),
        );
        return;
    }

    let meta = match std::fs::metadata(&path) {
        Ok(m) => m,
        Err(e) => {
            report.warn(
                "Python metadata",
                format!("cannot stat: {}", e),
                None,
            );
            return;
        }
    };

    let modified = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
    let age = SystemTime::now()
        .duration_since(modified)
        .unwrap_or_default();
    let days = age.as_secs() / 86_400;

    let max_days = config.general.auto_update_days as u64;
    if max_days > 0 && days > max_days {
        report.warn(
            "Python metadata",
            format!("{} days old (auto-update threshold: {})", days, max_days),
            Some("run `pvm update` to refresh"),
        );
    } else {
        report.ok("Python metadata", format!("{} days old", days));
    }
}

fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}
