//! Folder-picker abstraction.
//!
//! The production impl wraps `rfd::FileDialog::pick_folder`. Tests use a fake
//! that records the thread the picker ran on and optionally sleeps for a
//! configurable duration.

use std::path::PathBuf;

pub trait FolderPicker: Send + Sync {
    fn pick_folder(&self) -> Option<PathBuf>;
}

pub struct RfdPicker;

impl FolderPicker for RfdPicker {
    fn pick_folder(&self) -> Option<PathBuf> {
        rfd::FileDialog::new().pick_folder()
    }
}
