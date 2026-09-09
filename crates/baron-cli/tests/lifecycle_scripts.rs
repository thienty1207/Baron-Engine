use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use assert_cmd::cargo::cargo_bin;
use baron_core::release::{
    supported_release_target, write_release_metadata_with_signing_key, SUPPORTED_RELEASE_TARGETS,
};
use tempfile::tempdir;

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[cfg(target_os = "windows")]
fn powershell_command() -> ProcessCommand {
    if let Some(path) = std::env::var_os("BARON_TEST_POWERSHELL") {
        ProcessCommand::new(path)
    } else {
        ProcessCommand::new("powershell")
    }
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
    #[cfg(not(any(
        all(target_os = "windows", target_arch = "x86_64"),
        all(target_os = "linux", target_arch = "x86_64")
    )))]
    {
        panic!("unsupported Baron lifecycle test target; v5 publishes Windows x64 and Linux x64")
    }
}

fn package_current_binary(source_dir: &Path) -> PathBuf {
    let current = supported_release_target(current_target()).unwrap();
    let binary = cargo_bin("baron");

    for target in SUPPORTED_RELEASE_TARGETS {
        let archive = source_dir.join(target.archive_name(CURRENT_VERSION));
        if target.triple != current.triple {
            fs::write(&archive, format!("placeholder:{}", target.triple)).unwrap();
            continue;
        }

        #[cfg(target_os = "windows")]
        {
            let staging = source_dir.join("staging");
            fs::create_dir_all(&staging).unwrap();
            let staged_binary = staging.join("baron.exe");
            fs::copy(&binary, &staged_binary).unwrap();
            let command = format!(
                "Compress-Archive -LiteralPath '{}' -DestinationPath '{}' -Force",
                staged_binary.display().to_string().replace('\'', "''"),
                archive.display().to_string().replace('\'', "''")
            );
            let status = powershell_command()
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
    }

    write_release_metadata_with_signing_key(
        source_dir,
        CURRENT_VERSION,
        "0123456789abcdef0123456789abcdef01234567",
        "baron-debug-test-key",
        &[7_u8; 32],
    )
    .unwrap();
    source_dir.join(current.archive_name(CURRENT_VERSION))
}

fn installer_copy_with_test_key(source_dir: &Path) -> PathBuf {
    let source_name = if cfg!(target_os = "windows") {
        "installers/install.ps1"
    } else {
        "installers/install.sh"
    };
    let destination = source_dir.join(if cfg!(target_os = "windows") {
        "test-install.ps1"
    } else {
        "test-install.sh"
    });
    let mut script = fs::read_to_string(workspace_root().join(source_name)).unwrap();
    script = script.replace("baron-release-2026", "baron-debug-test-key");
    script = script.replace(
        "SBgrmtK5emhgD5e6UW664fXR3qogCjVPHtrxeVk2TcA=",
        "6kpsY+KcUgq+9VB7Ey7F+ZVHdq6+vnuSQh7qaRRG0iw=",
    );
    script = script.replace(
        "MCowBQYDK2VwAyEASBgrmtK5emhgD5e6UW664fXR3qogCjVPHtrxeVk2TcA=",
        "MCowBQYDK2VwAyEA6kpsY+KcUgq+9VB7Ey7F+ZVHdq6+vnuSQh7qaRRG0iw=",
    );
    fs::write(&destination, script).unwrap();
    destination
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
fn installer_scripts_require_pinned_signed_metadata_before_mutation() {
    let root = workspace_root();
    let powershell = fs::read_to_string(root.join("installers/install.ps1")).unwrap();
    let shell = fs::read_to_string(root.join("installers/install.sh")).unwrap();
    for script in [&powershell, &shell] {
        assert!(script.contains("baron-release-2026"));
        assert!(script.contains("SBgrmtK5emhgD5e6UW664fXR3qogCjVPHtrxeVk2TcA="));
        assert!(script.contains("release-manifest.sig"));
        assert!(script.contains("pkeyutl"));
        assert!(script.contains("refusing") || script.contains("Refusing"));
    }
    assert!(shell.contains("OpenSSL with Ed25519 support"));
    assert!(powershell.contains("OpenSSL with Ed25519 support"));
    assert!(shell.find("pkeyutl").unwrap() < shell.find("mv \"$staged_binary\"").unwrap());
    assert!(
        powershell.find("pkeyutl").unwrap()
            < powershell
                .find("Move-Item -LiteralPath $stagedBinary")
                .unwrap()
    );
}

#[test]
fn installer_scripts_resolve_latest_without_github_api_quota() {
    let root = workspace_root();
    let powershell = fs::read_to_string(root.join("installers/install.ps1")).unwrap();
    let shell = fs::read_to_string(root.join("installers/install.sh")).unwrap();

    for script in [&powershell, &shell] {
        assert!(script.contains("releases/latest/download/release-manifest.json"));
        assert!(!script.contains("api.github.com"));
    }
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
    let installer = installer_copy_with_test_key(&source);

    #[cfg(target_os = "windows")]
    let run = |action: &str| {
        powershell_command()
            .env("BARON_STATE_DIR", &state)
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                installer.to_str().unwrap(),
                "-Action",
                action,
                "-Version",
                CURRENT_VERSION,
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
            .arg(&installer)
            .args([
                "--action",
                action,
                "--version",
                CURRENT_VERSION,
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

#[cfg(target_os = "windows")]
#[test]
fn powershell_installer_makes_baron_available_in_the_current_session() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("release");
    let install = temp.path().join("install");
    let state = temp.path().join("state");
    fs::create_dir_all(&source).unwrap();
    package_current_binary(&source);
    let installer = installer_copy_with_test_key(&source);

    let script = format!(
        r#"
$oldUserPath = [Environment]::GetEnvironmentVariable("Path", "User")
try {{
    $env:Path = ($env:Path -split ';' | Where-Object {{ $_ -ne '{install}' }}) -join ';'
    & '{installer}' -Action install -Version {version} -InstallDir '{install}' -SourceDirectory '{source}' -StateDirectory '{state}'
    $command = Get-Command baron -ErrorAction Stop
    if ($command.Source -ne '{expected}') {{
        throw "baron resolved to '$($command.Source)' instead of '{expected}'"
    }}
    $version = (baron --version | Out-String).Trim()
    if ($version -ne 'baron {version}') {{
        throw "unexpected version: $version"
    }}
}} finally {{
    [Environment]::SetEnvironmentVariable("Path", $oldUserPath, "User")
}}
"#,
        installer = installer.display().to_string().replace('\'', "''"),
        install = install.display().to_string().replace('\'', "''"),
        source = source.display().to_string().replace('\'', "''"),
        state = state.display().to_string().replace('\'', "''"),
        expected = install
            .join("baron.exe")
            .display()
            .to_string()
            .replace('\'', "''"),
        version = CURRENT_VERSION,
    );

    let output = powershell_command()
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(target_os = "windows")]
#[test]
fn powershell_installer_rejects_signature_tampering_before_touching_existing_binary() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("release");
    let install = temp.path().join("install");
    let state = temp.path().join("state");
    fs::create_dir_all(&source).unwrap();
    package_current_binary(&source);
    let installer = installer_copy_with_test_key(&source);
    fs::create_dir_all(&install).unwrap();
    let existing = install.join("baron.exe");
    fs::write(&existing, b"existing-binary").unwrap();

    let mut signature: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(source.join("release-manifest.sig")).unwrap())
            .unwrap();
    signature["signature"] = serde_json::json!("00".repeat(64));
    fs::write(
        source.join("release-manifest.sig"),
        serde_json::to_string(&signature).unwrap(),
    )
    .unwrap();

    let output = powershell_command()
        .env("BARON_STATE_DIR", &state)
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            installer.to_str().unwrap(),
            "-Action",
            "install",
            "-Version",
            CURRENT_VERSION,
            "-InstallDir",
            install.to_str().unwrap(),
            "-SourceDirectory",
            source.to_str().unwrap(),
            "-NoPathUpdate",
        ])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(fs::read(&existing).unwrap(), b"existing-binary");
    assert!(!state.exists());
}

#[test]
fn native_installer_rejects_unsafe_version_before_installing() {
    let temp = tempdir().unwrap();
    let install = temp.path().join("install");
    let source = temp.path().join("release");
    let state = temp.path().join("state");
    fs::create_dir_all(&source).unwrap();

    #[cfg(target_os = "windows")]
    let output = powershell_command()
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
