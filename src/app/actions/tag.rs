use std::io;

use crate::app::{App, EntryLocation};
use crate::ui::{remove_all_trailing_tags, remove_last_trailing_tag};

use super::content_ops::{
    ContentTarget, execute_content_append, execute_content_operation, get_entry_content,
    set_entry_content,
};
use super::types::{Action, ActionDescription, StatusVisibility};

/// Target for tag operations (type alias for ContentTarget)
pub type TagTarget = ContentTarget;

/// Action to remove the last trailing tag from entries
pub struct RemoveLastTag {
    targets: Vec<TagTarget>,
}

impl RemoveLastTag {
    #[must_use]
    pub fn new(targets: Vec<TagTarget>) -> Self {
        Self { targets }
    }

    #[must_use]
    pub fn single(location: EntryLocation, original_content: String) -> Self {
        Self::new(vec![TagTarget {
            location,
            original_content,
        }])
    }
}

impl Action for RemoveLastTag {
    fn execute(&mut self, app: &mut App) -> io::Result<Box<dyn Action>> {
        for target in &self.targets {
            execute_content_operation(app, &target.location, remove_last_trailing_tag)?;
        }

        Ok(Box::new(RestoreContent::new(
            self.targets.clone(),
            TagOperation::RemoveLast,
        )))
    }

    fn description(&self) -> ActionDescription {
        let count = self.targets.len();
        if count == 1 {
            ActionDescription {
                past: "Removed tag".to_string(),
                past_reversed: "Restored tag".to_string(),
                visibility: StatusVisibility::Always,
            }
        } else {
            ActionDescription {
                past: format!("Removed tags from {} entries", count),
                past_reversed: format!("Restored tags on {} entries", count),
                visibility: StatusVisibility::Always,
            }
        }
    }
}

/// Action to remove all trailing tags from entries
pub struct RemoveAllTags {
    targets: Vec<TagTarget>,
}

impl RemoveAllTags {
    #[must_use]
    pub fn new(targets: Vec<TagTarget>) -> Self {
        Self { targets }
    }

    #[must_use]
    pub fn single(location: EntryLocation, original_content: String) -> Self {
        Self::new(vec![TagTarget {
            location,
            original_content,
        }])
    }
}

impl Action for RemoveAllTags {
    fn execute(&mut self, app: &mut App) -> io::Result<Box<dyn Action>> {
        for target in &self.targets {
            execute_content_operation(app, &target.location, remove_all_trailing_tags)?;
        }

        Ok(Box::new(RestoreContent::new(
            self.targets.clone(),
            TagOperation::RemoveAll,
        )))
    }

    fn description(&self) -> ActionDescription {
        let count = self.targets.len();
        if count == 1 {
            ActionDescription {
                past: "Removed all tags".to_string(),
                past_reversed: "Restored tags".to_string(),
                visibility: StatusVisibility::Always,
            }
        } else {
            ActionDescription {
                past: format!("Removed all tags from {} entries", count),
                past_reversed: format!("Restored tags on {} entries", count),
                visibility: StatusVisibility::Always,
            }
        }
    }
}

/// Action to append a tag to entries
pub struct AppendTag {
    targets: Vec<TagTarget>,
    tag: String,
}

impl AppendTag {
    #[must_use]
    pub fn new(targets: Vec<TagTarget>, tag: String) -> Self {
        Self { targets, tag }
    }

    #[must_use]
    pub fn single(location: EntryLocation, original_content: String, tag: String) -> Self {
        Self::new(
            vec![TagTarget {
                location,
                original_content,
            }],
            tag,
        )
    }
}

impl Action for AppendTag {
    fn execute(&mut self, app: &mut App) -> io::Result<Box<dyn Action>> {
        let suffix = format!(" #{}", self.tag);
        for target in &self.targets {
            execute_content_append(app, &target.location, &suffix)?;
        }

        Ok(Box::new(RestoreContent::new(
            self.targets.clone(),
            TagOperation::Append(self.tag.clone()),
        )))
    }

    fn description(&self) -> ActionDescription {
        let count = self.targets.len();
        if count == 1 {
            ActionDescription {
                past: format!("Added #{}", self.tag),
                past_reversed: format!("Removed #{}", self.tag),
                visibility: StatusVisibility::Always,
            }
        } else {
            ActionDescription {
                past: format!("Added #{} to {} entries", self.tag, count),
                past_reversed: format!("Removed #{} from {} entries", self.tag, count),
                visibility: StatusVisibility::Always,
            }
        }
    }
}

/// Which operation was performed (for redo)
#[derive(Clone)]
enum TagOperation {
    RemoveLast,
    RemoveAll,
    Append(String),
}

/// Action to restore original content (reverse of tag operations)
pub struct RestoreContent {
    targets: Vec<TagTarget>,
    operation: TagOperation,
}

impl RestoreContent {
    fn new(targets: Vec<TagTarget>, operation: TagOperation) -> Self {
        Self { targets, operation }
    }
}

impl Action for RestoreContent {
    fn execute(&mut self, app: &mut App) -> io::Result<Box<dyn Action>> {
        let mut new_targets = Vec::with_capacity(self.targets.len());
        for target in &self.targets {
            let current_content = get_entry_content(app, &target.location)?;
            set_entry_content(app, &target.location, &target.original_content)?;
            new_targets.push(ContentTarget::new(target.location.clone(), current_content));
        }

        let redo_action: Box<dyn Action> = match &self.operation {
            TagOperation::RemoveLast => Box::new(RemoveLastTag::new(new_targets)),
            TagOperation::RemoveAll => Box::new(RemoveAllTags::new(new_targets)),
            TagOperation::Append(tag) => Box::new(AppendTag::new(new_targets, tag.clone())),
        };

        Ok(redo_action)
    }

    fn description(&self) -> ActionDescription {
        let count = self.targets.len();
        match &self.operation {
            TagOperation::RemoveLast => {
                if count == 1 {
                    ActionDescription {
                        past: "Restored tag".to_string(),
                        past_reversed: "Removed tag".to_string(),
                        visibility: StatusVisibility::Always,
                    }
                } else {
                    ActionDescription {
                        past: format!("Restored tags on {} entries", count),
                        past_reversed: format!("Removed tags from {} entries", count),
                        visibility: StatusVisibility::Always,
                    }
                }
            }
            TagOperation::RemoveAll => {
                if count == 1 {
                    ActionDescription {
                        past: "Restored tags".to_string(),
                        past_reversed: "Removed all tags".to_string(),
                        visibility: StatusVisibility::Always,
                    }
                } else {
                    ActionDescription {
                        past: format!("Restored tags on {} entries", count),
                        past_reversed: format!("Removed all tags from {} entries", count),
                        visibility: StatusVisibility::Always,
                    }
                }
            }
            TagOperation::Append(tag) => {
                if count == 1 {
                    ActionDescription {
                        past: format!("Removed #{}", tag),
                        past_reversed: format!("Added #{}", tag),
                        visibility: StatusVisibility::Always,
                    }
                } else {
                    ActionDescription {
                        past: format!("Removed #{} from {} entries", tag, count),
                        past_reversed: format!("Added #{} to {} entries", tag, count),
                        visibility: StatusVisibility::Always,
                    }
                }
            }
        }
    }
}
