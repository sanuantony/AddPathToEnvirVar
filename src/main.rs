mod cli;
mod core;
#[cfg(windows)]
mod windows;

use clap::Parser;
use cli::{Cli, Commands};
use std::env;
use std::error::Error;
use std::io::{self, Write};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List { scope } => {
            run_if_windows(|| {
                let paths = windows::read_path(scope.is_system())?;
                core::print_paths(&paths);
                Ok(())
            })?;
        }

        Commands::Add {
            path,
            scope,
            dry_run,
            yes,
        } => {
            run_if_windows(|| {
                let system = scope.is_system();
                let target = core::resolve_input_path(&path)?;
                let mut paths = windows::read_path(system)?;

                if core::contains_path(&paths, &target) {
                    println!("Already present: {}", target.display());
                    return Ok(());
                }

                if !yes && !dry_run {
                    confirm_or_exit("This will modify PATH. Continue?")?;
                }

                if dry_run {
                    let mut preview = paths.clone();
                    core::add_path(&mut preview, &target)?;
                    println!("Dry run: would add {}", target.display());
                    core::print_paths(&preview);
                    return Ok(());
                }

                let backup_file = make_backup_name("add", system);
                core::backup_to_file(&paths, &backup_file)?;
                println!("Backup saved to {}", backup_file);

                core::add_path(&mut paths, &target)?;
                windows::write_path(&paths, system)?;
                println!("Added: {}", target.display());
                Ok(())
            })?;
        }

        Commands::Remove {
            path,
            scope,
            dry_run,
            yes,
        } => {
            run_if_windows(|| {
                let system = scope.is_system();
                let target = core::resolve_input_path(&path)?;
                let mut paths = windows::read_path(system)?;

                if !core::contains_path(&paths, &target) {
                    println!("Not found: {}", target.display());
                    return Ok(());
                }

                if !yes && !dry_run {
                    confirm_or_exit("This will modify PATH. Continue?")?;
                }

                if dry_run {
                    let mut preview = paths.clone();
                    core::remove_path(&mut preview, &target);
                    println!("Dry run: would remove {}", target.display());
                    core::print_paths(&preview);
                    return Ok(());
                }

                let backup_file = make_backup_name("remove", system);
                core::backup_to_file(&paths, &backup_file)?;
                println!("Backup saved to {}", backup_file);

                core::remove_path(&mut paths, &target);
                windows::write_path(&paths, system)?;
                println!("Removed: {}", target.display());
                Ok(())
            })?;
        }

        Commands::Backup { file, scope } => {
            run_if_windows(|| {
                let paths = windows::read_path(scope.is_system())?;
                core::backup_to_file(&paths, &file)?;
                println!("Backup saved to {}", file);
                Ok(())
            })?;
        }

        Commands::Restore {
            file,
            scope,
            dry_run,
            yes,
        } => {
            run_if_windows(|| {
                let system = scope.is_system();
                let paths = core::restore_from_file(&file)?;

                if !yes && !dry_run {
                    confirm_or_exit("This will replace PATH from the backup file. Continue?")?;
                }

                if dry_run {
                    println!("Dry run: would restore PATH from {}", file);
                    core::print_paths(&paths);
                    return Ok(());
                }

                let current = windows::read_path(system)?;
                let backup_file = make_backup_name("restore", system);
                core::backup_to_file(&current, &backup_file)?;
                println!("Backup saved to {}", backup_file);

                windows::write_path(&paths, system)?;
                println!("Restored PATH from {}", file);
                Ok(())
            })?;
        }
    }

    Ok(())
}

fn run_if_windows<F>(f: F) -> Result<(), Box<dyn Error>>
where
    F: FnOnce() -> Result<(), Box<dyn Error>>,
{
    #[cfg(windows)]
    {
        f()
    }

    #[cfg(not(windows))]
    {
        Err("This tool is currently implemented for Windows only.".into())
    }
}

fn confirm_or_exit(message: &str) -> Result<(), Box<dyn Error>> {
    print!("{message} [y/N]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let answer = input.trim().to_ascii_lowercase();
    if answer == "y" || answer == "yes" {
        Ok(())
    } else {
        println!("Cancelled.");
        process::exit(0);
    }
}

fn make_backup_name(action: &str, system: bool) -> String {
    let scope = if system { "system" } else { "user" };
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut dir = env::temp_dir();
    dir.push(format!("pathctl-{action}-{scope}-{ts}.txt"));
    dir.to_string_lossy().into_owned()
}