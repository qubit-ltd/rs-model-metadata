// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

static NEXT_TEMP_DIR_ID: AtomicU64 = AtomicU64::new(0);

pub(crate) struct TemporaryTargetDir {
    path: PathBuf,
}

impl TemporaryTargetDir {
    pub(crate) fn new(prefix: &str) -> Self {
        loop {
            let id = NEXT_TEMP_DIR_ID.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!("{prefix}-{}-{id}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => return Self { path },
                Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
                Err(error) => panic!(
                    "unable to create temporary Cargo target directory {}: {error}",
                    path.display()
                ),
            }
        }
    }

    #[must_use]
    #[inline]
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TemporaryTargetDir {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.path) {
            eprintln!(
                "unable to clean temporary Cargo target directory {}: {error}",
                self.path.display()
            );
        }
    }
}
