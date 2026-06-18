use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use assert_cmd::cargo::cargo_bin;
use baron_core::release::{sha256_file, supported_release_target};
use tempfile::tempdir;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn current_target() -> &'static str {
    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    {
        "x86_64-pc-windows-msvc"
    }
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        "x86_64-unknown-linux-gnu"
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        "x86_64-apple-darwin"
    }
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    {
        "aarch64-apple-darwin"
    }
}

fn package_current_binary(source_dir: &Path) -> PathBuf {
    let target = supported_release_target(current_target()).unwrap();
    let archive = source_dir.join(target.archive_name("3.0.0"));
    let binary = cargo_bin("baron");

    #[cfg(target_os = "windows")]
    {
        let command = format!(
            "Compress-Archive -LiteralPath '{}' -DestinationPath '{}' -Force",
            binary.display().to_string().replace('\'', "''"),
            archive.display().to_string().replace('\'', "''")
        );
        let status = ProcessCommand::new("powershell")
            .args(["-NoProfile", "-Command", &command])
            .status()
            .unwrap();
        assert!(status.success());
    }

    #[cfg(not(target_os = "windows"))]
    {
        let staging = source_dir.join("staging");
        fs::create_dir_all(&staging).unwrap();
        fs::copy(&binary, staging.join("baron")).unwrap();
        let status = ProcessCommand::new("tar")
            .args(["-czf"])
            .arg(&archive)
            .arg("-C")
            .arg(&staging)
            .arg("baron")
            .status()
            .unwrap();
        assert!(status.success());
    }

    let checksum = sha256_file(&archive).unwrap();
    fs::write(
        source_dir.join("SHA256SUMS"),
        format!(
            "{}  {}\n",
            checksum,
            archive.file_name().unwrap().to_string_lossy()
        ),
    )
    .unwrap();
    archive
}

#[test]
fn installer_scripts_enforce_checksum_and_data_safety_contracts() {
    let root = workspace_root();
    let powershell = fs::read_to_string(root.join("installers/install.ps1")).unwrap();
    let shell = fs::read_to_string(root.join("installers/install.sh")).unwrap();

    for script in [&powershell, &shell] {
        assert!(script.contains("SHA256SUMS"));
        assert!(script.contains("checksum"));
        assert!(script.contains("rollback"));
        assert!(script.contains("uninstall"));
        assert!(script.contains("BARON_RELEASE_BASE_URL"));
    }
    assert!(
        powershell.find("$actualChecksum").unwrap()
            < powershell
                .find("Move-Item -LiteralPath $stagedBinary")
                .unwrap()
    );
    assert!(powershell.contains("System.Security.Cryptography.SHA256"));
    assert!(!powershell.contains("Get-FileHash"));
    assert!(shell.find("checksum").unwrap() < shell.find("mv \"$staged_binary\"").unwrap());
    assert!(!powershell.contains("Projects\\"));
    assert!(!shell.contains("/Projects/"));
}

#[test]
fn native_installer_supports_install_update_rollback_and_uninstall() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("release");
    let install = temp.path().join("install");
    let state = temp.path().join("state");
    let data_sentinel = temp.path().join("vault-memory.md");
    fs::create_dir_all(&source).unwrap();
    fs::write(&data_sentinel, "must survive").unwrap();
    package_current_binary(&source);

    #[cfg(target_os = "windows")]
    let run = |action: &str| {
        ProcessCommand::new("powershell")
            .env("BARON_STATE_DIR", &state)
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                workspace_root()
                    .join("installers/install.ps1")
                    .to_str()
                    .unwrap(),
                "-Action",
                action,
                "-Version",
                "3.0.0",
                "-InstallDir",
                install.to_str().unwrap(),
                "-SourceDirectory",
                source.to_str().unwrap(),
                "-NoPathUpdate",
            ])
            .status()
            .unwrap()
    };

    #[cfg(not(target_os = "windows"))]
    let run = |action: &str| {
        ProcessCommand::new("sh")
            .env("BARON_STATE_DIR", &state)
            .arg(workspace_root().join("installers/install.sh"))
            .args([
                "--action",
                action,
                "--version",
                "3.0.0",
                "--install-dir",
                install.to_str().unwrap(),
                "--source-dir",
                source.to_str().unwrap(),
            ])
            .status()
            .unwrap()
    };

    assert!(run("install").success());
    let installed = install.join(if cfg!(target_os = "windows") {
        "baron.exe"
    } else {
        "baron"
    });
    assert!(installed.is_file());
    assert!(ProcessCommand::new(&installed)
        .arg("--version")
        .status()
        .unwrap()
        .success());

    assert!(run("update").success());
    assert!(run("rollback").success());
    assert!(installed.is_file());
    assert!(run("uninstall").success());
    assert!(!installed.exists());
    assert_eq!(fs::read_to_string(data_sentinel).unwrap(), "must survive");
}

#[test]
fn native_installer_rejects_unsafe_version_before_installing() {
    let temp = tempdir().unwrap();
    let install = temp.path().join("install");
    let source = temp.path().join("release");
    let state = temp.path().join("state");
    fs::create_dir_all(&source).unwrap();

    #[cfg(target_os = "windows")]
    let output = ProcessCommand::new("powershell")
        .env("BARON_STATE_DIR", &state)
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            workspace_root()
                .join("installers/install.ps1")
                .to_str()
                .unwrap(),
            "-Action",
            "install",
            "-Version",
            "../escape",
            "-InstallDir",
            install.to_str().unwrap(),
            "-SourceDirectory",
            source.to_str().unwrap(),
            "-NoPathUpdate",
        ])
        .output()
        .unwrap();

    #[cfg(not(target_os = "windows"))]
    let output = ProcessCommand::new("sh")
        .env("BARON_STATE_DIR", &state)
        .arg(workspace_root().join("installers/install.sh"))
        .args([
            "--action",
            "install",
            "--version",
            "../escape",
            "--install-dir",
            install.to_str().unwrap(),
            "--source-dir",
            source.to_str().unwrap(),
        ])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let message = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(message.contains("numeric major.minor.patch"));
    assert!(!install.join("baron").exists());
    assert!(!install.join("baron.exe").exists());
}
