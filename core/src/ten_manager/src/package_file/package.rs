//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::path::PathBuf;
use std::{fs::File, path::Path};

use anyhow::Result;
use flate2::write::GzEncoder;
use flate2::Compression;
use tar::Builder as TarBuilder;

pub fn tar_gz_files_to_file<P: AsRef<Path>>(
    files: Vec<P>,
    prefix: &Path,
    output_tar_gz_file_path: &String,
) -> Result<()> {
    let output_file = File::create(output_tar_gz_file_path)?;
    let mut gz_encoder = GzEncoder::new(output_file, Compression::default());

    {
        let mut tar_builder = TarBuilder::new(&mut gz_encoder);

        for path in files {
            let path = path.as_ref();
            let relative_path = path.strip_prefix(prefix)?.to_path_buf();

            if path.is_file() {
                tar_builder.append_path_with_name(path, &relative_path)?;
            } else {
                // If it is a directory and not the current one ".", then
                // package the directory.
                if !relative_path.as_os_str().is_empty()
                    && relative_path != PathBuf::from(".")
                {
                    tar_builder.append_dir_all(&relative_path, path)?;
                }
            }
        }

        // Finish the tar archive and retrieve the GzEncoder.
        tar_builder.finish()?;
    }

    // Properly finalize the GzEncoder.
    gz_encoder.finish()?;

    Ok(())
}
