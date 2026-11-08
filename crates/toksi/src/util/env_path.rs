// SPDX-FileCopyrightText: Copyright (c) 2026 Navegos. @DevelVitorF. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
// project: neo
// file: crates/toksi/src/util/env_path.rs
// created: 2026-11-08

/// Environment PATH Validation Mandate ("Rule of Test and Inform User")
///
/// Validates whether a required binary exists in system PATH.
/// Never hardcodes absolute paths; keeps configurations portable across machines.
/// On missing binary, returns a clean error string with exact remediation instructions.
pub fn check_executable_in_path(cmd: &str) -> Result<(), String> {
    let trimmed = cmd.trim();
    if trimmed.is_empty() {
        return Err("[ERROR] Executable name cannot be empty.".to_string());
    }

    if let Some(paths) = std::env::var_os("PATH").or_else(|| std::env::var_os("Path")) {
        for path in std::env::split_paths(&paths) {
            let full = path.join(trimmed);
            if full.is_file() {
                return Ok(());
            }
            #[cfg(windows)]
            {
                let low = trimmed.to_ascii_lowercase();
                if !low.ends_with(".exe") {
                    let with_exe = path.join(format!("{}.exe", trimmed));
                    if with_exe.is_file() {
                        return Ok(());
                    }
                }
                if !low.ends_with(".cmd") {
                    let with_cmd = path.join(format!("{}.cmd", trimmed));
                    if with_cmd.is_file() {
                        return Ok(());
                    }
                }
                if !low.ends_with(".bat") {
                    let with_bat = path.join(format!("{}.bat", trimmed));
                    if with_bat.is_file() {
                        return Ok(());
                    }
                }
            }
        }
    }

    Err(format!(
        "[ERROR] Required executable '{}' not found in system PATH. Please add '{}' to your PATH environment variable and retry.",
        trimmed, trimmed
    ))
}

/// Checks multiple executables and returns all missing binary errors.
pub fn check_executables_in_path(cmds: &[&str]) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    for &cmd in cmds {
        if let Err(e) = check_executable_in_path(cmd) {
            errors.push(e);
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_executable_in_path_nonexistent() {
        let res = check_executable_in_path("definitely_not_existing_binary_xyz_12345");
        assert!(res.is_err());
        let err = res.err().unwrap_or_default();
        assert!(err.contains("[ERROR] Required executable 'definitely_not_existing_binary_xyz_12345' not found in system PATH."));
        assert!(err.contains("Please add 'definitely_not_existing_binary_xyz_12345' to your PATH environment variable and retry."));
    }

    #[test]
    fn test_check_executable_in_path_cargo() {
        // Cargo should be in PATH when running cargo tests
        let res = check_executable_in_path("cargo");
        assert!(res.is_ok());
    }

    #[test]
    fn test_check_executables_in_path() {
        let res = check_executables_in_path(&["cargo"]);
        assert!(res.is_ok());

        let res_err = check_executables_in_path(&["definitely_fake_bin_1", "definitely_fake_bin_2"]);
        assert!(res_err.is_err());
        let errs = res_err.err().unwrap_or_default();
        assert_eq!(errs.len(), 2);
    }
}
