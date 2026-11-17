use std::path::{Path, PathBuf};
use std::process::Command;

use mae_domain::DomainError;

pub(super) fn find_desktop_manifest(target_dir: &Path) -> Result<PathBuf, DomainError> {
    let crates_dir = target_dir.join("crates");
    let entries = std::fs::read_dir(&crates_dir).map_err(|error| {
        DomainError::PortError(format!(
            "Visual verification requires a generated modular Slint desktop target under {}: {error}",
            crates_dir.display()
        ))
    })?;
    let mut candidates = entries
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .ends_with("-desktop-app")
        })
        .map(|entry| entry.path().join("Cargo.toml"))
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    candidates.sort();
    match candidates.as_slice() {
        [manifest] => Ok(manifest.clone()),
        [] => Err(DomainError::PortError(format!(
            "Visual verification requires a generated modular Slint desktop target under {}",
            crates_dir.display()
        ))),
        _ => Err(DomainError::PortError(format!(
            "Visual verification found multiple Slint desktop targets under {}; select one target workspace",
            crates_dir.display()
        ))),
    }
}

pub(super) fn capture_native_route(
    target_dir: &Path,
    manifest: &Path,
    route: &str,
    output: &Path,
) -> Result<(), String> {
    let result = Command::new("cargo")
        .args([
            "run",
            "--quiet",
            "--manifest-path",
            manifest.to_str().unwrap_or_default(),
            "--no-default-features",
            "--features",
            "headless-test",
        ])
        .current_dir(target_dir)
        .env("MAE_VISUAL_ROUTE", route)
        .env("MAE_VISUAL_OUTPUT", output)
        .output()
        .map_err(|error| format!("cannot launch native Slint target with cargo: {error}"))?;
    if !result.status.success() {
        return Err(format!(
            "native Slint capture failed: {}",
            String::from_utf8_lossy(&result.stderr).trim()
        ));
    }
    if !output.is_file() {
        return Err(format!(
            "native Slint target exited successfully without writing {}",
            output.display()
        ));
    }
    Ok(())
}
