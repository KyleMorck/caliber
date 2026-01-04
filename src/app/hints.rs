use std::sync::LazyLock;

/// What kind of hints to display
#[derive(Clone, Debug, PartialEq)]
pub enum HintContext {
    /// No hints to display
    Inactive,
    /// Tag hints from current journal
    Tags {
        prefix: String,
        matches: Vec<String>,
    },
    /// Command hints (static list)
    Commands {
        prefix: String,
        matches: Vec<&'static CommandHint>,
    },
    /// Filter type hints (!tasks, !notes, etc.)
    FilterTypes {
        prefix: String,
        matches: Vec<&'static FilterTypeHint>,
    },
    /// Date operation hints (@before:, @after:, @overdue)
    DateOps {
        prefix: String,
        matches: Vec<&'static DateOpHint>,
    },
    /// Negation hints (not:#, not:!, not:word)
    Negation {
        prefix: String,
        matches: Vec<&'static NegationHint>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct CommandHint {
    pub command: &'static str,
    pub aliases: &'static [&'static str],
    pub description: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FilterTypeHint {
    pub syntax: &'static str,
    pub description: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DateOpHint {
    pub syntax: &'static str,
    pub description: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NegationHint {
    pub syntax: &'static str,
    pub description: &'static str,
}

pub static COMMAND_HINTS: LazyLock<Vec<CommandHint>> = LazyLock::new(|| {
    vec![
        CommandHint {
            command: "date",
            aliases: &["d"],
            description: "Go to date (MM/DD)",
        },
        CommandHint {
            command: "global",
            aliases: &["g"],
            description: "Switch to Global journal",
        },
        CommandHint {
            command: "project",
            aliases: &["p"],
            description: "Project [init|default|path]",
        },
        CommandHint {
            command: "config",
            aliases: &["c"],
            description: "Config [reload]",
        },
        CommandHint {
            command: "quit",
            aliases: &["q"],
            description: "Quit application",
        },
    ]
});

pub static FILTER_TYPE_HINTS: LazyLock<Vec<FilterTypeHint>> = LazyLock::new(|| {
    vec![
        FilterTypeHint {
            syntax: "!tasks",
            description: "Incomplete tasks",
        },
        FilterTypeHint {
            syntax: "!t",
            description: "Tasks (short)",
        },
        FilterTypeHint {
            syntax: "!tasks/done",
            description: "Completed tasks",
        },
        FilterTypeHint {
            syntax: "!tasks/all",
            description: "All tasks",
        },
        FilterTypeHint {
            syntax: "!notes",
            description: "Notes only",
        },
        FilterTypeHint {
            syntax: "!n",
            description: "Notes (short)",
        },
        FilterTypeHint {
            syntax: "!events",
            description: "Events only",
        },
        FilterTypeHint {
            syntax: "!e",
            description: "Events (short)",
        },
    ]
});

pub static DATE_OP_HINTS: LazyLock<Vec<DateOpHint>> = LazyLock::new(|| {
    vec![
        DateOpHint {
            syntax: "@before:",
            description: "Entries before date",
        },
        DateOpHint {
            syntax: "@after:",
            description: "Entries after date",
        },
        DateOpHint {
            syntax: "@overdue",
            description: "Entries with past @date",
        },
    ]
});

pub static NEGATION_HINTS: LazyLock<Vec<NegationHint>> = LazyLock::new(|| {
    vec![
        NegationHint {
            syntax: "not:#",
            description: "Exclude tag",
        },
        NegationHint {
            syntax: "not:!",
            description: "Exclude entry type",
        },
        NegationHint {
            syntax: "not:",
            description: "Exclude text",
        },
    ]
});

/// Which input context we're computing hints for
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HintMode {
    /// Command mode (:)
    Command,
    /// Filter query mode (/)
    Filter,
    /// Entry editing mode
    Entry,
}

impl HintContext {
    /// Compute hints based on current input buffer and mode
    #[must_use]
    pub fn compute(input: &str, mode: HintMode, journal_tags: &[String]) -> Self {
        match mode {
            HintMode::Command => Self::compute_command_hints(input),
            HintMode::Filter => Self::compute_filter_hints(input, journal_tags),
            HintMode::Entry => Self::compute_entry_hints(input, journal_tags),
        }
    }

    fn compute_command_hints(input: &str) -> Self {
        let prefix = input.trim();

        let matches: Vec<&'static CommandHint> = COMMAND_HINTS
            .iter()
            .filter(|h| {
                h.command.starts_with(prefix) || h.aliases.iter().any(|a| a.starts_with(prefix))
            })
            .collect();

        if matches.is_empty() {
            Self::Inactive
        } else {
            Self::Commands {
                prefix: prefix.to_string(),
                matches,
            }
        }
    }

    fn compute_tag_hints(input: &str, journal_tags: &[String]) -> Self {
        let current_token = input.split_whitespace().last().unwrap_or("");

        if let Some(tag_prefix) = current_token.strip_prefix('#') {
            let matches: Vec<String> = journal_tags
                .iter()
                .filter(|t| t.to_lowercase().starts_with(&tag_prefix.to_lowercase()))
                .cloned()
                .collect();

            if matches.is_empty()
                || (matches.len() == 1 && matches[0].eq_ignore_ascii_case(tag_prefix))
            {
                return Self::Inactive;
            }
            return Self::Tags {
                prefix: tag_prefix.to_string(),
                matches,
            };
        }

        Self::Inactive
    }

    fn compute_entry_hints(input: &str, journal_tags: &[String]) -> Self {
        Self::compute_tag_hints(input, journal_tags)
    }

    fn compute_filter_hints(input: &str, journal_tags: &[String]) -> Self {
        let current_token = input.split_whitespace().last().unwrap_or("");

        if current_token.starts_with('#') {
            return Self::compute_tag_hints(input, journal_tags);
        }

        if let Some(type_prefix) = current_token.strip_prefix('!') {
            let matches: Vec<&'static FilterTypeHint> = FILTER_TYPE_HINTS
                .iter()
                .filter(|h| h.syntax[1..].starts_with(type_prefix))
                .collect();

            if matches.is_empty() {
                return Self::Inactive;
            }
            return Self::FilterTypes {
                prefix: type_prefix.to_string(),
                matches,
            };
        }

        if let Some(date_prefix) = current_token.strip_prefix('@') {
            let matches: Vec<&'static DateOpHint> = DATE_OP_HINTS
                .iter()
                .filter(|h| h.syntax[1..].starts_with(date_prefix))
                .collect();

            if matches.is_empty() {
                return Self::Inactive;
            }
            return Self::DateOps {
                prefix: date_prefix.to_string(),
                matches,
            };
        }

        if let Some(neg_prefix) = current_token.strip_prefix("not:") {
            let matches: Vec<&'static NegationHint> = NEGATION_HINTS
                .iter()
                .filter(|h| h.syntax[4..].starts_with(neg_prefix))
                .collect();

            if matches.is_empty() {
                return Self::Inactive;
            }
            return Self::Negation {
                prefix: neg_prefix.to_string(),
                matches,
            };
        }

        Self::Inactive
    }

    #[must_use]
    pub fn first_completion(&self) -> Option<String> {
        match self {
            Self::Inactive => None,
            Self::Tags { prefix, matches } => {
                matches.first().map(|tag| tag[prefix.len()..].to_string())
            }
            Self::Commands { prefix, matches } => {
                matches.first().map(|hint| hint.command[prefix.len()..].to_string())
            }
            Self::FilterTypes { prefix, matches } => {
                matches.first().map(|hint| hint.syntax[1 + prefix.len()..].to_string())
            }
            Self::DateOps { prefix, matches } => {
                matches.first().map(|hint| hint.syntax[1 + prefix.len()..].to_string())
            }
            Self::Negation { prefix, matches } => {
                matches.first().map(|hint| hint.syntax[4 + prefix.len()..].to_string())
            }
        }
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        !matches!(self, Self::Inactive)
    }
}

