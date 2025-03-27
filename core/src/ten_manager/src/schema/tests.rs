//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use serde_json;

use super::{validate_designer_config, validate_tman_config};

// Test minimal valid config.
#[test]
fn test_minimal_valid_config() {
    let config_json = serde_json::json!({
        "registry": {
            "default": {
                "index": "https://test-registry.com"
            }
        }
    });

    let result = validate_tman_config(&config_json);
    assert!(
        result.is_ok(),
        "Should validate a minimal config: {:?}",
        result
    );
}

// Test full valid config.
#[test]
fn test_full_valid_config() {
    let config_json = serde_json::json!({
        "registry": {
            "default": {
                "index": "https://test-registry.com"
            }
        },
        "admin_token": "admin-token",
        "user_token": "user-token",
        "enable_package_cache": true,
        "designer": {
            "logviewer_line_size": 2000
        }
    });

    let result = validate_tman_config(&config_json);
    assert!(
        result.is_ok(),
        "Should validate a full config: {:?}",
        result
    );
}

// Test case for missing required field.
#[test]
fn test_invalid_missing_required_field() {
    let config_json = serde_json::json!({
        // Missing registry field.
        "admin_token": "admin-token",
        "user_token": "user-token"
    });

    let result = validate_tman_config(&config_json);
    assert!(result.is_err(), "Should fail when missing required field");
}

// Test case for invalid field type.
#[test]
fn test_invalid_field_type() {
    let config_json = serde_json::json!({
        "registry": "not-an-object", // Should be an object.
        "admin_token": "admin-token"
    });

    let result = validate_tman_config(&config_json);
    assert!(result.is_err(), "Should fail with invalid field type");
}

// Test case for valid designer config.
#[test]
fn test_valid_designer_config() {
    let designer_json = serde_json::json!({
        "logviewer_line_size": 2000
    });

    let result = validate_designer_config(&designer_json);
    assert!(
        result.is_ok(),
        "Should validate a valid designer config: {:?}",
        result
    );
}

// Test case for designer config validation - minimum value constraint.
#[test]
fn test_designer_config_minimum_constraint() {
    let designer_json = serde_json::json!({
        "logviewer_line_size": 50 // Below minimum of 100.
    });

    let result = validate_designer_config(&designer_json);
    assert!(
        result.is_err(),
        "Should fail when logviewer_line_size is below minimum"
    );
}

// Test case for designer config validation - missing required field.
#[test]
fn test_designer_config_missing_required_field() {
    let designer_json = serde_json::json!({
        // Missing logviewer_line_size field
    });

    let result = validate_designer_config(&designer_json);
    assert!(
        result.is_err(),
        "Should fail when logviewer_line_size is missing"
    );
}

// Test case for designer config validation - invalid field type.
#[test]
fn test_designer_config_invalid_field_type() {
    let designer_json = serde_json::json!({
        "logviewer_line_size": "not-a-number" // Should be an integer.
    });

    let result = validate_designer_config(&designer_json);
    assert!(
        result.is_err(),
        "Should fail when logviewer_line_size is not an integer"
    );
}

// Test case for designer section in full config.
#[test]
fn test_designer_section_in_full_config() {
    let config_json = serde_json::json!({
        "registry": {
            "default": {
                "index": "https://test-registry.com"
            }
        },
        "designer": {
            "logviewer_line_size": 50 // Below minimum of 100.
        }
    });

    let result = validate_tman_config(&config_json);
    assert!(
        result.is_err(),
        "Should fail when designer section has invalid values"
    );
}

// Test case for designer section with default values.
#[test]
fn test_designer_section_with_default_values() {
    // Use the default value defined in the JSON schema (1000).
    let config_json = serde_json::json!({
        "registry": {
            "default": {
                "index": "https://test-registry.com"
            }
        },
        "designer": {
            // logviewer_line_size not specified, should use default.
        }
    });

    // Note: The jsonschema validator does not automatically apply defaults.
    // This test is more to document the behavior than to validate it.
    let result = validate_tman_config(&config_json);
    assert!(
        result.is_err(),
        "Schema validation doesn't apply defaults automatically"
    );
}

// Test case for additional properties.
#[test]
fn test_additional_properties() {
    let config_json = serde_json::json!({
        "registry": {
            "default": {
                "index": "https://test-registry.com"
            }
        },
        "unknown_field": "some value", // Additional field.
        "designer": {
            "logviewer_line_size": 2000,
            "unknown_designer_field": true // Additional field in designer section.
        }
    });

    let result = validate_tman_config(&config_json);
    // By default, JSON Schema allows additional properties.
    assert!(
        result.is_ok(),
        "Should allow additional properties by default"
    );

    let designer_json = serde_json::json!({
        "logviewer_line_size": 2000,
        "unknown_designer_field": true // Additional field in designer section.
    });

    let result = validate_designer_config(&designer_json);
    assert!(
        result.is_ok(),
        "Should allow additional properties in designer schema"
    );
}

// Test case for getting designer schema function.
#[test]
fn test_get_designer_schema() {
    use super::get_designer_schema;

    let schema_str = get_designer_schema();
    assert!(
        !schema_str.is_empty(),
        "Designer schema should not be empty"
    );

    // Verify the returned JSON is valid.
    let schema_result = serde_json::from_str::<serde_json::Value>(schema_str);
    assert!(
        schema_result.is_ok(),
        "Designer schema should be valid JSON"
    );

    // Verify it contains expected key fields.
    let schema_json = schema_result.unwrap();
    assert!(schema_json.is_object(), "Schema should be a JSON object");
    assert!(
        schema_json.get("type").is_some(),
        "Schema should have 'type' field"
    );
    assert!(
        schema_json.get("properties").is_some(),
        "Schema should have 'properties' field"
    );

    // Verify it contains logviewer_line_size property.
    if let Some(props) = schema_json.get("properties") {
        assert!(
            props.get("logviewer_line_size").is_some(),
            "Schema should have logviewer_line_size property"
        );
    } else {
        panic!("Properties field not found in schema");
    }
}
