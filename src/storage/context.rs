use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JournalSlot {
    Global,
    Project,
}

struct JournalContext {
    global_path: PathBuf,
    project_path: Option<PathBuf>,
    active: JournalSlot,
}

static JOURNAL_CONTEXT: RwLock<Option<JournalContext>> = RwLock::new(None);

pub fn set_journal_context(global: PathBuf, project: Option<PathBuf>, active: JournalSlot) {
    if let Ok(mut guard) = JOURNAL_CONTEXT.write() {
        *guard = Some(JournalContext {
            global_path: global,
            project_path: project,
            active,
        });
    }
}

#[must_use]
pub fn get_active_slot() -> JournalSlot {
    JOURNAL_CONTEXT
        .read()
        .ok()
        .and_then(|guard| guard.as_ref().map(|ctx| ctx.active))
        .unwrap_or(JournalSlot::Global)
}

pub fn set_active_slot(slot: JournalSlot) {
    if let Ok(mut guard) = JOURNAL_CONTEXT.write()
        && let Some(ctx) = guard.as_mut()
    {
        ctx.active = slot;
    }
}

#[must_use]
pub fn get_project_path() -> Option<PathBuf> {
    JOURNAL_CONTEXT
        .read()
        .ok()
        .and_then(|guard| guard.as_ref().and_then(|ctx| ctx.project_path.clone()))
}

pub fn set_project_path(path: PathBuf) {
    if let Ok(mut guard) = JOURNAL_CONTEXT.write()
        && let Some(ctx) = guard.as_mut()
    {
        ctx.project_path = Some(path);
    }
}

/// Resets project path to the auto-detected default (.caliber/journal.md)
pub fn reset_project_path() {
    if let Ok(mut guard) = JOURNAL_CONTEXT.write()
        && let Some(ctx) = guard.as_mut()
    {
        ctx.project_path = detect_project_journal();
    }
}

/// Resets the journal context (for testing only)
pub fn reset_journal_context() {
    if let Ok(mut guard) = JOURNAL_CONTEXT.write() {
        *guard = None;
    }
}

#[must_use]
pub fn get_active_journal_path() -> PathBuf {
    JOURNAL_CONTEXT
        .read()
        .ok()
        .and_then(|guard| {
            guard.as_ref().map(|ctx| match ctx.active {
                JournalSlot::Global => ctx.global_path.clone(),
                JournalSlot::Project => ctx
                    .project_path
                    .clone()
                    .unwrap_or_else(|| ctx.global_path.clone()),
            })
        })
        .unwrap_or_else(crate::config::get_default_journal_path)
}

/// Detects if we're in a git repository and returns the project root path.
#[must_use]
pub fn find_git_root() -> Option<PathBuf> {
    let mut current = std::env::current_dir().ok()?;
    loop {
        if current.join(".git").exists() {
            return Some(current);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

/// Detects if a project journal exists and returns its path.
/// Returns Some(path) if .caliber/journal.md exists, None otherwise.
#[must_use]
pub fn detect_project_journal() -> Option<PathBuf> {
    // First check for git root
    if let Some(root) = find_git_root() {
        let project_journal = root.join(".caliber").join("journal.md");
        if project_journal.exists() {
            return Some(project_journal);
        }
        return None;
    }

    // Fallback: check current directory for .caliber/
    let cwd = std::env::current_dir().ok()?;
    let project_journal = cwd.join(".caliber").join("journal.md");
    if project_journal.exists() {
        return Some(project_journal);
    }

    None
}

/// Creates a project journal at .caliber/journal.md in the git root.
pub fn create_project_journal() -> io::Result<PathBuf> {
    let root = find_git_root()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Not in a git repository"))?;

    let caliber_dir = root.join(".caliber");
    fs::create_dir_all(&caliber_dir)?;

    let journal_path = caliber_dir.join("journal.md");
    if !journal_path.exists() {
        fs::write(&journal_path, "")?;
    }

    Ok(journal_path)
}

/// Adds .caliber/ to .gitignore if not already present.
pub fn add_caliber_to_gitignore() -> io::Result<()> {
    let root = find_git_root()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Not in a git repository"))?;

    let gitignore_path = root.join(".gitignore");
    let entry = ".caliber/";

    if gitignore_path.exists() {
        let content = fs::read_to_string(&gitignore_path)?;
        if content.lines().any(|line| line.trim() == entry) {
            return Ok(());
        }
        let mut new_content = content;
        if !new_content.ends_with('\n') && !new_content.is_empty() {
            new_content.push('\n');
        }
        new_content.push_str(entry);
        new_content.push('\n');
        fs::write(&gitignore_path, new_content)?;
    } else {
        fs::write(&gitignore_path, format!("{entry}\n"))?;
    }

    Ok(())
}
