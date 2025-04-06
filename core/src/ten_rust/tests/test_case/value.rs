//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#[cfg(test)]
mod tests {
    use serde_json::json;
    use ten_rust::value::{create_value_from_json, create_value_from_json_str};

    #[test]
    fn test_create_value_from_json_str() {
        let json_str = r#"{"name": "test", "value": 42}"#;
        let result = create_value_from_json_str(json_str);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_value_from_json() {
        let json_value = json!({
            "name": "test",
            "value": 42
        });
        let result = create_value_from_json(&json_value);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_value_from_invalid_json_str() {
        let json_str = r#"{"name": "test", "value": }"#; // Invalid JSON
        let result = create_value_from_json_str(json_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_value_drop() {
        // This test ensures that the Drop trait is properly implemented
        // by creating and then dropping a TenValue without causing any issues
        let json_str = r#"{"name": "test", "value": 42}"#;
        {
            let _value = create_value_from_json_str(json_str).unwrap();
            // _value will be dropped at the end of this scope
        }
        // If we reach here without segfaults, the Drop implementation is
        // working
    }
}
