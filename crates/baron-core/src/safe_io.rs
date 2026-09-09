//! Small filesystem primitives shared by Baron-managed writers.
//!
//! The callers still own their path-policy and transaction semantics. This
//! module only provides explicit file reads, preserve-first replacement, safe
//! directory creation, and one project-scoped mutation lock.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions, Permissions};
use std::io::{self, Seek, SeekFrom, Write};
use std::marker::PhantomData;
use std::path::{Component, Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::{self, ThreadId};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};

const MUTATION_LOCK_RELATIVE: &str = ".baron/.baron-mutation.lock";
const DEFAULT_LOCK_TIMEOUT: Duration = Duration::from_secs(5);
const LOCK_RETRY_INTERVAL: Duration = Duration::from_millis(20);
const MAX_TEMP_ATTEMPTS: usize = 128;

static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);
static LOCKS: OnceLock<Mutex<HashMap<PathBuf, LockState>>> = OnceLock::new();

/// Reads a regular file as bytes.
///
/// `Ok(None)` means the file is absent. Any other filesystem error, a
/// directory, or a symlink is returned as an error so callers cannot mistake
/// a failed read for an empty file.
pub fn read_bytes(path: impl AsRef<Path>) -> Result<Option<Vec<u8>>> {
    let path = path.as_ref();
    let Some(parent_exists) = inspect_existing_parent_chain(path)? else {
        return Ok(None);
    };
    if !parent_exists {
        return Ok(None);
    }
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(error)
                .with_context(|| format!("Could not inspect file: {}", path.display()))
        }
    };
    if metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
        bail!(
            "Refusing to read symlink or reparse point as a managed regular file: {}",
            path.display()
        );
    }
    if !metadata.is_file() {
        bail!(
            "Expected a regular file but found another filesystem entry: {}",
            path.display()
        );
    }
    fs::read(path)
        .with_context(|| format!("Could not read file: {}", path.display()))
        .map(Some)
}

/// Reads a regular file as UTF-8 text with explicit missing-file semantics.
pub fn read_text(path: impl AsRef<Path>) -> Result<Option<String>> {
    let path = path.as_ref();
    let Some(bytes) = read_bytes(path)? else {
        return Ok(None);
    };
    String::from_utf8(bytes)
        .with_context(|| format!("File is not valid UTF-8: {}", path.display()))
        .map(Some)
}

/// Reads a required UTF-8 file and gives a specific missing-file diagnostic.
pub fn read_text_required(path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    read_text(path)?.ok_or_else(|| anyhow::anyhow!("Required file is missing: {}", path.display()))
}

/// Creates a directory and every missing ancestor after validating that every
/// existing component is a real directory rather than a symlink or reparse
/// point. Parent traversal is rejected.
pub fn ensure_directory_chain(path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    if path.as_os_str().is_empty() {
        bail!("Directory path cannot be empty");
    }
    let created = inspect_directory_chain(path, true)?;
    if !created {
        bail!("Directory path is unavailable: {}", path.display());
    }
    Ok(())
}

/// Computes the project-scoped mutation lock path after resolving the project
/// root. This is also used by diagnostics and deterministic lock fixtures.
pub fn project_lock_path(repo_root: impl AsRef<Path>) -> Result<PathBuf> {
    let root = canonical_project_root(repo_root.as_ref())?;
    Ok(root.join(MUTATION_LOCK_RELATIVE))
}

/// Acquires the bounded project mutation lock with the default timeout.
pub fn acquire_project_lock(repo_root: impl AsRef<Path>) -> Result<ProjectMutationLock> {
    acquire_project_lock_with_timeout(repo_root, DEFAULT_LOCK_TIMEOUT)
}

/// Acquires the project mutation lock with a caller-selected bounded wait.
///
/// The lock is an OS-level advisory lock on a stable project path. The marker
/// text is diagnostic only; it is never used as permission to delete a live
/// lock. A process crash releases the OS lock, so an old marker is recoverable
/// once the operating system proves the file is available.
pub fn acquire_project_lock_with_timeout(
    repo_root: impl AsRef<Path>,
    timeout: Duration,
) -> Result<ProjectMutationLock> {
    let root = canonical_project_root(repo_root.as_ref())?;
    let key = root.clone();
    let path = root.join(MUTATION_LOCK_RELATIVE);
    let parent = path
        .parent()
        .context("Project mutation lock has no parent directory")?;
    ensure_directory_chain(parent)?;

    let current_thread = thread::current().id();
    if let Some(file) = reentrant_lock(&key, current_thread) {
        return Ok(ProjectMutationLock {
            key,
            file,
            _thread_affine: PhantomData,
        });
    }

    let started = Instant::now();
    loop {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)
            .with_context(|| format!("Could not open Baron mutation lock: {}", path.display()))?;
        match try_lock_file(&file) {
            Ok(true) => {
                write_lock_marker(&file)?;
                let file = Arc::new(file);
                register_lock(key.clone(), current_thread, Arc::clone(&file));
                return Ok(ProjectMutationLock {
                    key,
                    file,
                    _thread_affine: PhantomData,
                });
            }
            Ok(false) => {
                let owner = lock_owner_diagnostic(&path);
                if started.elapsed() >= timeout {
                    bail!(
                        "Timed out waiting for Baron mutation lock `{}`. {}",
                        path.display(),
                        owner
                    );
                }
                let remaining = timeout.saturating_sub(started.elapsed());
                thread::sleep(LOCK_RETRY_INTERVAL.min(remaining));
            }
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("Could not acquire Baron mutation lock: {}", path.display())
                })
            }
        }
    }
}

/// A reentrant project mutation lock. All guards for one thread share the
/// underlying locked file; the OS lock is released only by the final guard.
pub struct ProjectMutationLock {
    key: PathBuf,
    file: Arc<File>,
    // A guard must be dropped by the thread that acquired it. Keeping a
    // thread-affine marker prevents accidental cross-thread moves from
    // leaving the process-local reentrancy registry holding a dead owner.
    _thread_affine: PhantomData<Rc<()>>,
}

impl std::fmt::Debug for ProjectMutationLock {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ProjectMutationLock")
            .field("key", &self.key)
            .finish_non_exhaustive()
    }
}

impl Drop for ProjectMutationLock {
    fn drop(&mut self) {
        let current_thread = thread::current().id();
        let should_unlock = {
            let mut locks = lock_registry()
                .lock()
                .unwrap_or_else(|poison| poison.into_inner());
            match locks.get_mut(&self.key) {
                Some(state) if state.owner == current_thread && state.depth > 1 => {
                    state.depth -= 1;
                    false
                }
                Some(state) if state.owner == current_thread => {
                    locks.remove(&self.key);
                    true
                }
                _ => false,
            }
        };
        if should_unlock {
            let _ = unlock_file(&self.file);
        }
    }
}

/// Replaces a regular file using same-directory staging and preserve-first
/// activation. Existing bytes are left in place until the staged file is
/// ready. Identical bytes are a no-op.
pub fn replace_file(path: impl AsRef<Path>, content: &[u8]) -> Result<()> {
    replace_file_impl(path.as_ref(), content, false)
}

/// UTF-8 convenience wrapper around [`replace_file`].
pub fn replace_text(path: impl AsRef<Path>, content: &str) -> Result<()> {
    replace_file(path, content.as_bytes())
}

/// Appends bytes to a regular file and durably flushes the file before
/// returning. Callers that need cross-process serialization must hold the
/// project mutation lock while using this primitive.
pub fn append_bytes(path: impl AsRef<Path>, content: &[u8]) -> Result<()> {
    let path = path.as_ref();
    let parent = path
        .parent()
        .context("Append target has no parent directory")?;
    ensure_directory_chain(parent)?;
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || is_reparse_point(&metadata) => {
            bail!(
                "Append target cannot be a symlink or reparse point: {}",
                path.display()
            )
        }
        Ok(metadata) if !metadata.is_file() => {
            bail!("Append target is not a regular file: {}", path.display())
        }
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error)
                .with_context(|| format!("Could not inspect append target: {}", path.display()))
        }
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("Could not open append target: {}", path.display()))?;
    file.write_all(content)
        .with_context(|| format!("Could not append to file: {}", path.display()))?;
    file.flush()
        .with_context(|| format!("Could not flush append target: {}", path.display()))?;
    file.sync_all()
        .with_context(|| format!("Could not sync append target: {}", path.display()))?;
    sync_parent_directory(parent)
}

/// UTF-8 convenience wrapper around [`append_bytes`].
pub fn append_text(path: impl AsRef<Path>, content: &str) -> Result<()> {
    append_bytes(path, content.as_bytes())
}

fn replace_file_impl(path: &Path, content: &[u8], fail_before_activation: bool) -> Result<()> {
    #[cfg(not(test))]
    let _ = fail_before_activation;
    let parent = path
        .parent()
        .context("Replacement target has no parent directory")?;
    ensure_directory_chain(parent)?;
    if let Some(existing) = read_bytes(path)? {
        if existing == content {
            return Ok(());
        }
    }

    let permissions = existing_permissions(path)?;
    let (temporary, mut file) = create_unique_temporary(path)?;
    let result = (|| {
        file.write_all(content)
            .with_context(|| format!("Could not stage replacement: {}", temporary.display()))?;
        file.flush()?;
        file.sync_all()
            .with_context(|| format!("Could not flush replacement: {}", temporary.display()))?;
        drop(file);
        if let Some(permissions) = permissions {
            fs::set_permissions(&temporary, permissions).with_context(|| {
                format!(
                    "Could not preserve file permissions: {}",
                    temporary.display()
                )
            })?;
        }
        ensure_directory_chain(parent)?;
        let _ = existing_permissions(path)?;
        #[cfg(test)]
        if fail_before_activation {
            bail!("Injected interrupted replacement before activation");
        }
        activate_replacement(&temporary, path)
    })();
    if let Err(error) = result {
        let cleanup = fs::remove_file(&temporary);
        if let Err(cleanup_error) = cleanup {
            if cleanup_error.kind() != io::ErrorKind::NotFound {
                return Err(error.context(format!(
                    "Replacement failed and temporary cleanup also failed for {}: {}",
                    temporary.display(),
                    cleanup_error
                )));
            }
        }
        return Err(error);
    }
    sync_parent_directory(parent)?;
    Ok(())
}

fn canonical_project_root(path: &Path) -> Result<PathBuf> {
    let root = path
        .canonicalize()
        .with_context(|| format!("Could not resolve project root: {}", path.display()))?;
    let metadata = fs::metadata(&root)
        .with_context(|| format!("Could not inspect project root: {}", root.display()))?;
    if !metadata.is_dir() {
        bail!("Project root is not a directory: {}", root.display());
    }
    Ok(root)
}

fn inspect_existing_parent_chain(path: &Path) -> Result<Option<bool>> {
    let Some(parent) = path.parent() else {
        return Ok(Some(true));
    };
    inspect_directory_chain(parent, false).map(Some)
}

fn inspect_directory_chain(path: &Path, create: bool) -> Result<bool> {
    let mut current = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => current.push(prefix.as_os_str()),
            Component::RootDir => current.push(std::path::MAIN_SEPARATOR_STR),
            Component::CurDir => {}
            Component::ParentDir => bail!(
                "Path escapes its allowed directory boundary: {}",
                path.display()
            ),
            Component::Normal(part) => {
                current.push(part);
                match fs::symlink_metadata(&current) {
                    Ok(metadata) => {
                        if metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
                            bail!(
                                "Path cannot traverse a symlink or junction/reparse point: {}",
                                current.display()
                            );
                        }
                        if !metadata.is_dir() {
                            bail!("Path component is not a directory: {}", current.display());
                        }
                    }
                    Err(error) if error.kind() == io::ErrorKind::NotFound && create => {
                        fs::create_dir(&current).or_else(|create_error| {
                            if create_error.kind() == io::ErrorKind::AlreadyExists {
                                Ok(())
                            } else {
                                Err(create_error)
                            }
                        })?;
                        let metadata = fs::symlink_metadata(&current).with_context(|| {
                            format!("Could not verify created directory: {}", current.display())
                        })?;
                        if metadata.file_type().is_symlink()
                            || is_reparse_point(&metadata)
                            || !metadata.is_dir()
                        {
                            bail!("Created directory is unsafe: {}", current.display());
                        }
                    }
                    Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
                    Err(error) => {
                        return Err(error).with_context(|| {
                            format!("Could not inspect directory: {}", current.display())
                        })
                    }
                }
            }
        }
    }
    Ok(true)
}

fn existing_permissions(path: &Path) -> Result<Option<Permissions>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
                bail!(
                    "Replacement target cannot be a symlink or reparse point: {}",
                    path.display()
                );
            }
            if !metadata.is_file() {
                bail!(
                    "Replacement target is not a regular file: {}",
                    path.display()
                );
            }
            Ok(Some(metadata.permissions()))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error)
            .with_context(|| format!("Could not inspect replacement target: {}", path.display())),
    }
}

fn create_unique_temporary(path: &Path) -> Result<(PathBuf, File)> {
    // Keep the historical deterministic name reserved as a hard failure. This
    // prevents an old interrupted writer's sentinel from ever being reused or
    // overwritten while all new attempts use collision-resistant names.
    let legacy = path.with_extension("baron-tmp");
    match fs::symlink_metadata(&legacy) {
        Ok(_) => {
            bail!(
                "Replacement temporary-name collision at {}; refusing to touch the existing file",
                legacy.display()
            )
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error).with_context(|| {
                format!(
                    "Could not inspect replacement staging path: {}",
                    legacy.display()
                )
            })
        }
    }
    let parent = path.parent().context("Replacement target has no parent")?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("baron-file");
    let process = std::process::id();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    for _ in 0..MAX_TEMP_ATTEMPTS {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(
            ".{name}.baron-tmp-{process}-{timestamp}-{sequence}"
        ));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(file) => return Ok((candidate, file)),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(error).with_context(|| {
                    format!(
                        "Could not create replacement staging file: {}",
                        candidate.display()
                    )
                })
            }
        }
    }
    bail!(
        "Could not allocate a unique replacement staging file beside {}",
        path.display()
    )
}

#[cfg(unix)]
fn activate_replacement(temporary: &Path, path: &Path) -> Result<()> {
    fs::rename(temporary, path)
        .with_context(|| format!("Could not activate replacement: {}", path.display()))
}

#[cfg(windows)]
fn activate_replacement(temporary: &Path, path: &Path) -> Result<()> {
    if !path.exists() {
        return fs::rename(temporary, path)
            .with_context(|| format!("Could not activate replacement: {}", path.display()));
    }
    let backup = create_unique_backup(path)?;
    fs::rename(path, &backup).with_context(|| {
        format!(
            "Could not stage the existing target for Windows replacement: {}",
            path.display()
        )
    })?;
    match fs::rename(temporary, path) {
        Ok(()) => {
            let _ = fs::remove_file(backup);
            Ok(())
        }
        Err(error) => {
            let restore = if !path.exists() {
                fs::rename(&backup, path)
            } else {
                Ok(())
            };
            match restore {
                Ok(()) => Err(error).with_context(|| {
                    format!("Could not activate replacement: {}", path.display())
                }),
                Err(restore_error) => Err(error).context(format!(
                    "Could not activate replacement for {} and could not restore the prior target: {}",
                    path.display(), restore_error
                )),
            }
        }
    }
}

#[cfg(windows)]
fn create_unique_backup(path: &Path) -> Result<PathBuf> {
    let parent = path.parent().context("Replacement target has no parent")?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("baron-file");
    for _ in 0..MAX_TEMP_ATTEMPTS {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(".{name}.baron-backup-{sequence}"));
        if fs::symlink_metadata(&candidate).is_err() {
            return Ok(candidate);
        }
    }
    bail!(
        "Could not allocate a unique Windows replacement backup beside {}",
        path.display()
    )
}

fn sync_parent_directory(parent: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        let directory = File::open(parent).with_context(|| {
            format!(
                "Could not open replacement parent for sync: {}",
                parent.display()
            )
        })?;
        directory
            .sync_all()
            .with_context(|| format!("Could not sync replacement parent: {}", parent.display()))?;
    }
    #[cfg(not(unix))]
    let _ = parent;
    Ok(())
}

fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x0400 != 0
    }
    #[cfg(not(windows))]
    {
        let _ = metadata;
        false
    }
}

struct LockState {
    owner: ThreadId,
    depth: usize,
    file: Arc<File>,
}

fn lock_registry() -> &'static Mutex<HashMap<PathBuf, LockState>> {
    LOCKS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn reentrant_lock(key: &Path, owner: ThreadId) -> Option<Arc<File>> {
    let mut locks = lock_registry()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    let state = locks.get_mut(key)?;
    if state.owner != owner {
        return None;
    }
    state.depth += 1;
    Some(Arc::clone(&state.file))
}

fn register_lock(key: PathBuf, owner: ThreadId, file: Arc<File>) {
    let mut locks = lock_registry()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());
    locks.insert(
        key,
        LockState {
            owner,
            depth: 1,
            file,
        },
    );
}

fn write_lock_marker(file: &File) -> Result<()> {
    let marker = format!(
        "pid={} thread={:?} acquired_unix_ms={}\n",
        std::process::id(),
        thread::current().id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .unwrap_or_default()
    );
    let mut file = file;
    file.set_len(0)?;
    file.seek(SeekFrom::Start(0))?;
    file.write_all(marker.as_bytes())?;
    file.flush()?;
    file.sync_all()?;
    Ok(())
}

fn lock_owner_diagnostic(path: &Path) -> String {
    match fs::read(path) {
        Ok(bytes) if bytes.is_empty() => {
            "The lock is held by another process or thread (owner marker is empty).".to_string()
        }
        Ok(bytes) => format!(
            "The lock is held by another process or thread; marker: {}",
            String::from_utf8_lossy(&bytes).trim()
        ),
        Err(error) => format!(
            "The lock is held by another process or thread; owner marker unavailable: {error}"
        ),
    }
}

#[cfg(unix)]
fn try_lock_file(file: &File) -> Result<bool> {
    use std::os::fd::AsRawFd;

    let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    if result == 0 {
        return Ok(true);
    }
    let error = io::Error::last_os_error();
    let code = error.raw_os_error();
    if code == Some(libc::EAGAIN) || code == Some(libc::EWOULDBLOCK) {
        Ok(false)
    } else {
        Err(error).context("Could not lock Baron mutation file")
    }
}

#[cfg(unix)]
fn unlock_file(file: &File) -> Result<()> {
    use std::os::fd::AsRawFd;
    let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_UN) };
    if result == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error()).context("Could not unlock Baron mutation file")
    }
}

#[cfg(windows)]
fn try_lock_file(file: &File) -> Result<bool> {
    use std::mem::zeroed;
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        LockFileEx, LOCKFILE_EXCLUSIVE_LOCK, LOCKFILE_FAIL_IMMEDIATELY,
    };
    use windows_sys::Win32::System::IO::OVERLAPPED;

    let mut overlapped: OVERLAPPED = unsafe { zeroed() };
    let result = unsafe {
        LockFileEx(
            file.as_raw_handle(),
            LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
            0,
            1,
            0,
            &mut overlapped,
        )
    };
    if result != 0 {
        return Ok(true);
    }
    let error = io::Error::last_os_error();
    if error.raw_os_error() == Some(33) {
        Ok(false)
    } else {
        Err(error).context("Could not lock Baron mutation file")
    }
}

#[cfg(windows)]
fn unlock_file(file: &File) -> Result<()> {
    use std::mem::zeroed;
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::UnlockFileEx;
    use windows_sys::Win32::System::IO::OVERLAPPED;

    let mut overlapped: OVERLAPPED = unsafe { zeroed() };
    let result = unsafe { UnlockFileEx(file.as_raw_handle(), 0, 1, 0, &mut overlapped) };
    if result != 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error()).context("Could not unlock Baron mutation file")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use tempfile::tempdir;

    #[test]
    fn read_text_distinguishes_missing_content_invalid_utf8_and_directory() {
        let temp = tempdir().unwrap();
        assert_eq!(read_text(temp.path().join("missing.txt")).unwrap(), None);
        fs::write(temp.path().join("value.txt"), "value").unwrap();
        assert_eq!(
            read_text(temp.path().join("value.txt")).unwrap().as_deref(),
            Some("value")
        );
        fs::write(temp.path().join("invalid.txt"), [0xff, 0xfe]).unwrap();
        assert!(read_text(temp.path().join("invalid.txt")).is_err());
        fs::create_dir(temp.path().join("directory")).unwrap();
        assert!(read_text(temp.path().join("directory")).is_err());
    }

    #[test]
    fn interrupted_replacement_keeps_destination_and_cleans_staging() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("value.txt");
        fs::write(&target, "before").unwrap();
        let error = replace_file_impl(&target, b"after", true).unwrap_err();
        assert!(error.to_string().contains("interrupted replacement"));
        assert_eq!(fs::read_to_string(&target).unwrap(), "before");
        let staged = fs::read_dir(temp.path())
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_name().to_string_lossy().contains("baron-tmp-"))
            .collect::<Vec<_>>();
        assert!(staged.is_empty(), "staging artifacts remain: {staged:?}");
    }

    #[test]
    fn replacement_is_a_no_op_for_identical_bytes() {
        let temp = tempdir().unwrap();
        let target = temp.path().join("value.txt");
        fs::write(&target, "same").unwrap();
        let before = fs::metadata(&target).unwrap();
        replace_file(&target, b"same").unwrap();
        let after = fs::metadata(&target).unwrap();
        assert_eq!(fs::read_to_string(target).unwrap(), "same");
        assert_eq!(
            before.permissions().readonly(),
            after.permissions().readonly()
        );
    }

    #[test]
    fn nested_lock_is_reentrant_and_stale_marker_is_recoverable() {
        let temp = tempdir().unwrap();
        let first = acquire_project_lock(temp.path()).unwrap();
        let nested = acquire_project_lock(temp.path()).unwrap();
        drop(nested);
        drop(first);

        let path = project_lock_path(temp.path()).unwrap();
        fs::write(&path, "pid=999999 thread=dead acquired_unix_ms=0\n").unwrap();
        let recovered =
            acquire_project_lock_with_timeout(temp.path(), Duration::from_millis(100)).unwrap();
        drop(recovered);
        assert!(fs::read_to_string(path)
            .unwrap()
            .contains(&format!("pid={}", std::process::id())));
    }

    #[test]
    fn lock_contention_is_bounded_and_releases_after_owner_drop() {
        let temp = tempdir().unwrap();
        let owner = acquire_project_lock(temp.path()).unwrap();
        let barrier = Arc::new(Barrier::new(2));
        let ready = Arc::clone(&barrier);
        let repo = temp.path().to_path_buf();
        let contender = std::thread::spawn(move || {
            ready.wait();
            acquire_project_lock_with_timeout(&repo, Duration::from_millis(120))
                .unwrap_err()
                .to_string()
        });
        barrier.wait();
        let error = contender.join().unwrap();
        assert!(error.contains("Timed out waiting for Baron mutation lock"));
        drop(owner);
        acquire_project_lock_with_timeout(temp.path(), Duration::from_millis(120)).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn directory_creation_rejects_symlink_escape() {
        use std::os::unix::fs::symlink;
        let temp = tempdir().unwrap();
        let outside = temp.path().join("outside");
        let link = temp.path().join("link");
        fs::create_dir(&outside).unwrap();
        symlink(&outside, &link).unwrap();
        assert!(ensure_directory_chain(link.join("escape")).is_err());
    }
}
