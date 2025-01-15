//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::fs::{self, File};
use std::io::{self};
use std::path::Path;

use anyhow::Result;
use flate2::read::GzDecoder;
use tar::Archive as TarArchive;

use crate::install::installed_paths::{sort_installed_paths, InstalledPaths};
use crate::install::template::extract_and_process_tar_gz_template_part;

fn extract_and_process_tar_gz_normal_part(
    tar_gz_path: &str,
    output_dir: &str,
) -> Result<InstalledPaths> {
    // Open the package file.
    let file = File::open(tar_gz_path)?;
    let mut archive = TarArchive::new(GzDecoder::new(file));

    let mut installed_paths = InstalledPaths { paths: Vec::new() };

    // Iterate through each entry in the package file.
    for entry_result in archive.entries()? {
        let mut entry = entry_result?;
        let entry_path = entry.path()?;
        let entry_str = entry_path.to_string_lossy().to_string();

        let is_template = entry_str.ends_with(".tent");
        if is_template {
            // Do not process template files while handling the normal part;
            // leave them to be handled separately when specifically processing
            // the template part.
            continue;
        }

        let out_path = Path::new(output_dir).join(&entry_str);

        // Check if the entry is a file or directory.
        if entry.header().entry_type().is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            // Ensure that the parent directory exists.
            if let Some(p) = out_path.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }

            let mut out_file = File::create(&out_path)?;
            io::copy(&mut entry, &mut out_file)?;

            #[cfg(unix)]
            {
                // Set file permissions if it's a Unix system.
                use std::os::unix::fs::PermissionsExt;

                // Set file permissions if it's a Unix system.
                if let Ok(mode) = entry.header().mode() {
                    fs::set_permissions(
                        &out_path,
                        fs::Permissions::from_mode(mode),
                    )?;
                }
            }
        }

        let relative_path = out_path
            .strip_prefix(output_dir)?
            .to_str()
            .unwrap()
            .to_string();
        installed_paths.paths.push(relative_path);
    }

    Ok(installed_paths)
}

pub fn extract_and_process_tar_gz(
    tar_gz_path: &str,
    output_dir: &str,
    template_ctx: Option<&serde_json::Value>,
) -> Result<InstalledPaths> {
    let mut installed_paths =
        extract_and_process_tar_gz_normal_part(tar_gz_path, output_dir)?;

    if let Some(template_ctx) = template_ctx {
        // In the 'template' mode.
        extract_and_process_tar_gz_template_part(
            tar_gz_path,
            output_dir,
            template_ctx,
        )?;

        let tent_paths: Vec<_> = installed_paths
            .paths
            .iter()
            .filter(|p| p.ends_with(".tent"))
            .cloned()
            .collect();

        for tent_path in tent_paths {
            installed_paths.paths.retain(|p| p != &tent_path);

            let original_path = tent_path.trim_end_matches(".tent").to_string();
            if !installed_paths.paths.contains(&original_path) {
                installed_paths.paths.push(original_path);
            }
        }
    }

    // Custom sort: directories after their contents.
    sort_installed_paths(&mut installed_paths);

    Ok(installed_paths)
}
