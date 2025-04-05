//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use std::fs;
    use std::path::PathBuf;
    use ten_manager::pkg_info::manifest::to_file::*;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestStruct {
        pub name: String,
        pub value: i32,
    }

    #[test]
    fn test_save_and_load() {
        // Create a temporary file path
        let temp_file = PathBuf::from("test_manifest.json");

        // Test data
        let test_data = TestStruct {
            name: "test".to_string(),
            value: 42,
        };

        // Save to file
        let save_result = save_to_file(&test_data, &temp_file);
        assert!(save_result.is_ok());

        // Verify the file exists
        assert!(temp_file.exists());

        // Load from file
        let load_result: Result<TestStruct, _> = load_from_file(&temp_file);
        assert!(load_result.is_ok());

        // Verify the data matches
        assert_eq!(load_result.unwrap(), test_data);

        // Clean up
        let _ = fs::remove_file(temp_file);
    }
}
