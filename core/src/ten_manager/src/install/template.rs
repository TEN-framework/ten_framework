//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;

use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use handlebars::Handlebars;
use tar::Archive as TarArchive;

fn render_template(
    template: &str,
    template_ctx: &serde_json::Value,
) -> Result<String> {
    let reg = Handlebars::new();
    let rendered = reg
        .render_template(template, template_ctx)
        .with_context(|| "Failed to render template")?;
    Ok(rendered)
}

pub fn extract_and_process_tar_gz_template_part(
    pkg_path: &str,
    output_dir: &str,
    template_ctx: &serde_json::Value,
) -> Result<()> {
    let file = File::open(pkg_path)?;
    let mut archive = TarArchive::new(GzDecoder::new(file));

    // Iterate through each entry in the package file.
    for entry_result in archive.entries()? {
        let mut entry = entry_result?;
        let entry_path = entry.path()?;
        let entry_str = entry_path.to_string_lossy().to_string();

        let is_template = entry_str.ends_with(".tent");
        if is_template {
            // Note that we set the target file path to be written as
            // `output_dir + entry_str`, ensuring that the final relative path
            // written locally matches the path inside the tar file.
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

                // Read the entry inside the TAR file and write its contents to
                // `out_path`.
                let mut out_file = File::create(&out_path)?;
                io::copy(&mut entry, &mut out_file)?;
            }

            // Now, `out_path` already contains the `xxx.tent` file. We need to
            // first read its text content, perform template rendering, and then
            // write it back.

            let mut contents = String::new();
            {
                let mut out_file_for_read = File::open(&out_path)?;
                out_file_for_read.read_to_string(&mut contents)?;
            }
            let rendered = render_template(&contents, template_ctx)?;

            // Removes the .tent extension.
            let new_out_path = out_path.with_extension("");

            let mut new_out_file = File::create(&new_out_path)?;
            new_out_file.write_all(rendered.as_bytes())?;

            #[cfg(unix)]
            {
                // Set file permissions if it's a Unix system.
                use std::os::unix::fs::PermissionsExt;
                if let Ok(mode) = entry.header().mode() {
                    fs::set_permissions(
                        &new_out_path,
                        fs::Permissions::from_mode(mode),
                    )?;
                }
            }

            // Remove the original file.
            fs::remove_file(out_path)?;
        }
    }

    Ok(())
}
