use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub fn print_paths(paths: &[PathBuf]) {
    for p in paths {
        println!("{}", p.display());
    }
}

pub fn resolve_input_path(input: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let input = input.trim();
    if input.is_empty() {
        return Err("Empty path is not allowed".into());
    }

    let expanded = expand_tilde_and_env(input)?;
    let path = Path::new(&expanded);

    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()?.join(path)
    };

    match abs.canonicalize() {
        Ok(p) => Ok(p),
        Err(_) => Ok(abs),
    }
}

fn expand_tilde_and_env(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut s = input.to_string();

    if s == "~" || s.starts_with("~/") || s.starts_with("~\\") {
        let home = env::var("USERPROFILE").or_else(|_| env::var("HOME"))?;
        s = format!("{home}{}", &s[1..]);
    }

    s = expand_windows_env_vars(&s);
    Ok(s)
}

fn expand_windows_env_vars(input: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '%' {
            if let Some(end) = chars[i + 1..].iter().position(|&c| c == '%') {
                let name: String = chars[i + 1..i + 1 + end].iter().collect();
                if let Ok(val) = env::var(&name) {
                    out.push_str(&val);
                } else {
                    out.push('%');
                    out.push_str(&name);
                    out.push('%');
                }
                i += end + 2;
                continue;
            }
        }

        out.push(chars[i]);
        i += 1;
    }

    out
}

pub fn normalize_for_compare(path: &Path) -> String {
    let mut s = path.to_string_lossy().to_string();

    while s.ends_with('\\') || s.ends_with('/') {
        if s.len() <= 3 {
            break;
        }
        s.pop();
    }

    #[cfg(windows)]
    {
        s = s.to_lowercase();
    }

    s
}

pub fn contains_path(paths: &[PathBuf], target: &Path) -> bool {
    let target_key = normalize_for_compare(target);
    paths.iter().any(|p| normalize_for_compare(p) == target_key)
}

pub fn add_path(paths: &mut Vec<PathBuf>, target: &Path) -> Result<bool, Box<dyn std::error::Error>> {
    if !target.exists() {
        return Err(format!("Path does not exist: {}", target.display()).into());
    }

    if !target.is_dir() {
        return Err(format!("Not a directory: {}", target.display()).into());
    }

    if contains_path(paths, target) {
        return Ok(false);
    }

    paths.push(target.to_path_buf());
    Ok(true)
}

pub fn remove_path(paths: &mut Vec<PathBuf>, target: &Path) -> bool {
    let target_key = normalize_for_compare(target);
    let before = paths.len();

    paths.retain(|p| normalize_for_compare(p) != target_key);

    before != paths.len()
}

pub fn backup_to_file(paths: &[PathBuf], file: &str) -> Result<(), Box<dyn std::error::Error>> {
    let joined = env::join_paths(paths)?;
    fs::write(file, joined.to_string_lossy().as_ref())?;
    Ok(())
}

pub fn restore_from_file(file: &str) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file)?;
    let content = content.trim_end_matches('\n').trim_end_matches('\r');
    let os = std::ffi::OsString::from(content);
    Ok(env::split_paths(&os).collect())
}