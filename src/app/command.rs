use std::io;

use super::{App, ConfirmContext, InputMode};

impl App {
    pub fn execute_command(&mut self) -> io::Result<()> {
        let cmd = self.command_buffer.content().to_string();
        self.command_buffer.clear();
        let parts: Vec<&str> = cmd.trim().splitn(2, ' ').collect();
        let command = parts.first().copied().unwrap_or("");
        let arg = parts.get(1).copied().unwrap_or("").trim();

        match command {
            "q" | "quit" => {
                self.save();
                self.should_quit = true;
            }
            "d" | "date" => {
                if arg.is_empty() {
                    self.set_status("Usage: :date MM/DD");
                } else if let Some(date) = Self::parse_goto_date(arg) {
                    self.goto_day(date)?;
                } else {
                    self.set_status(format!("Invalid date: {arg}"));
                }
            }
            "c" | "config" => {
                if arg == "reload" {
                    self.reload_config()?;
                } else {
                    self.set_status("Usage: :config reload");
                }
            }
            "g" | "global" => {
                self.switch_to_global()?;
            }
            "p" | "project" => match arg {
                "" => {
                    if self.journal_context.project_path().is_some() {
                        self.switch_to_project()?;
                    } else if self.in_git_repo {
                        self.input_mode = InputMode::Confirm(ConfirmContext::CreateProjectJournal);
                        return Ok(());
                    } else {
                        self.set_status("Not in a git repository - no project journal available");
                    }
                }
                "init" => {
                    if self.journal_context.project_path().is_some() {
                        // Project exists - ensure config.toml exists too
                        if let Some(root) = crate::storage::find_git_root() {
                            let config_path = root.join(".caliber").join("config.toml");
                            if !config_path.exists() {
                                std::fs::write(&config_path, "")?;
                                self.set_status("Project config created");
                            } else {
                                self.set_status("Project already initialized");
                            }
                        }
                    } else if self.in_git_repo {
                        self.input_mode = InputMode::Confirm(ConfirmContext::CreateProjectJournal);
                        return Ok(());
                    } else {
                        let cwd = std::env::current_dir()?;
                        let caliber_dir = cwd.join(".caliber");
                        std::fs::create_dir_all(&caliber_dir)?;
                        let journal_path = caliber_dir.join("journal.md");
                        if !journal_path.exists() {
                            std::fs::write(&journal_path, "")?;
                        }
                        let config_path = caliber_dir.join("config.toml");
                        if !config_path.exists() {
                            std::fs::write(&config_path, "")?;
                        }
                        self.journal_context.set_project_path(journal_path);
                        self.switch_to_project()?;
                        self.set_status("Project journal created");
                    }
                }
                "default" => {
                    self.journal_context.reset_project_path();
                    if self.journal_context.project_path().is_some() {
                        self.switch_to_project()?;
                    } else {
                        self.set_status("No default project journal found");
                    }
                }
                path if path.ends_with(".md") => {
                    self.open_journal(path)?;
                }
                _ => {
                    self.set_status("Usage: :project [init|default|path.md]");
                }
            },
            _ => {
                if !command.is_empty() {
                    self.set_status(format!("Unknown command: {command}"));
                }
            }
        }
        self.input_mode = InputMode::Normal;
        Ok(())
    }
}
