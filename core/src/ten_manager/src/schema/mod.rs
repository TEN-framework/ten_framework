//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
mod definition;

pub use definition::TMAN_CONFIG_SCHEMA;

use anyhow::Result;
use json5;
use jsonschema::Validator;
use serde_json;

/// Validates a JSON value against the TmanConfig schema.
pub fn validate_tman_config(config_json: &serde_json::Value) -> Result<()> {
    let schema_json: serde_json::Value = json5::from_str(TMAN_CONFIG_SCHEMA)?;
    let validator = Validator::new(&schema_json)?;

    if let Err(error) = validator.validate(config_json) {
        return Err(anyhow::anyhow!("Config validation failed: {}", error));
    }

    Ok(())
}
