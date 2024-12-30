//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
mod bindings;
mod definition;

extern crate anyhow;
extern crate jsonschema;
extern crate serde_json;

use std::path::Path;

use anyhow::Result;
use json5;
use jsonschema::Validator;

fn load_schema(content: &str) -> Validator {
    // Use json5 to strip comments from the json string.
    let schema_json: serde_json::Value = json5::from_str(content).unwrap();

    jsonschema::validator_for(&schema_json).unwrap()
}

fn validate_json_object(
    json: &serde_json::Value,
    schema_str: &str,
) -> Result<()> {
    let validator = load_schema(schema_str);

    match validator.validate(json) {
        Ok(()) => Ok(()),
        Err(_) => {
            let mut msgs = String::new();
            for error in validator.iter_errors(json) {
                msgs.push_str(&format!(
                    "{} @ {}\n",
                    error, error.instance_path
                ));
            }
            Err(anyhow::anyhow!("{}", msgs))
        }
    }
}

pub fn ten_validate_manifest_json_string(data: &str) -> Result<()> {
    let manifest_json: serde_json::Value = serde_json::from_str(data)?;
    validate_json_object(&manifest_json, definition::MANIFEST_SCHEMA_DEFINITION)
}

pub fn ten_validate_manifest_json_file(file_path: &str) -> Result<()> {
    let file = std::fs::File::open(file_path)?;
    let reader = std::io::BufReader::new(file);
    let manifest_json: serde_json::Value = serde_json::from_reader(reader)?;

    validate_json_object(&manifest_json, definition::MANIFEST_SCHEMA_DEFINITION)
}

pub fn validate_manifest_lock_json_string(data: &str) -> Result<()> {
    let manifest_lock_json: serde_json::Value = serde_json::from_str(data)?;
    validate_json_object(
        &manifest_lock_json,
        definition::MANIFEST_LOCK_SCHEMA_DEFINITION,
    )
}

pub fn validate_manifest_lock_json_file(file_path: &str) -> Result<()> {
    let file = std::fs::File::open(file_path)?;
    let reader = std::io::BufReader::new(file);
    let manifest_lock_json: serde_json::Value =
        serde_json::from_reader(reader)?;

    validate_json_object(
        &manifest_lock_json,
        definition::MANIFEST_LOCK_SCHEMA_DEFINITION,
    )
}

pub fn ten_validate_property_json_string(data: &str) -> Result<()> {
    let property_json: serde_json::Value = serde_json::from_str(data)?;
    validate_json_object(&property_json, definition::PROPERTY_SCHEMA_DEFINITION)
}

pub fn ten_validate_property_json_file<P: AsRef<Path>>(
    file_path: P,
) -> Result<()> {
    let file = std::fs::File::open(file_path)?;
    let reader = std::io::BufReader::new(file);
    let property_json: serde_json::Value = serde_json::from_reader(reader)?;

    validate_json_object(&property_json, definition::PROPERTY_SCHEMA_DEFINITION)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_default_cpp_app() {
        let manifest = r#"
        {
          "type": "app",
          "name": "default_app_cpp",
          "version": "0.1.0",
          "dependencies": [],
          "api": {}
        }
        "#;
        let result = ten_validate_manifest_json_string(manifest);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_dependencies_normal() {
        let manifest = r#"
        {
          "type": "app",
          "name": "default_app_cpp",
          "version": "0.1.0",
          "dependencies": [{
            "type": "system",
            "name": "ten_runtime",
            "version": "0.6.0"
          }],
          "api": {}
        }
        "#;
        let result = ten_validate_manifest_json_string(manifest);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_dependencies_with_path() {
        let manifest = r#"
        {
          "type": "app",
          "name": "default_app_cpp",
          "version": "0.1.0",
          "dependencies": [{
            "path": "path/to/dependency"
          }],
          "api": {}
        }
        "#;
        let result = ten_validate_manifest_json_string(manifest);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_dependencies_normal_and_with_path() {
        let manifest = r#"
        {
          "type": "app",
          "name": "default_app_cpp",
          "version": "0.1.0",
          "dependencies": [{
            "type": "system",
            "name": "ten_runtime",
            "version": "0.6.0"
          },{
            "path": "path/to/dependency"
          }],
          "api": {}
        }
        "#;
        let result = ten_validate_manifest_json_string(manifest);
        assert!(result.is_ok());
    }

    #[test]
    fn test_only_app_needs_predefined_graphs() {
        let manifest = r#"
        {
          "type": "extension",
          "name": "default_extension_cpp",
          "version": "0.1.0",
          "dependencies": []
        }
        "#;
        let result = ten_validate_manifest_json_string(manifest);
        assert!(result.is_ok());
    }

    #[test]
    fn test_only_app_needs_predefined_graphs_supports() {
        let manifest = r#"
        {
          "type": "extension",
          "name": "default_extension_cpp",
          "version": "0.1.0",
          "dependencies": [],
          "supports": []
        }
        "#;
        let result = ten_validate_manifest_json_string(manifest);
        assert!(result.is_ok());
    }

    #[test]
    fn test_only_app_needs_predefined_graphs_supports_with_content() {
        let manifest = r#"
        {
          "type": "extension",
          "name": "default_extension_cpp",
          "version": "0.1.0",
          "dependencies": [],
          "supports": [{"os": "linux", "arch": "x64"}]
        }
        "#;
        let result = ten_validate_manifest_json_string(manifest);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_type() {
        let manifest = r#"
        {
          "type": "invalid",
          "name": "default_app_cpp",
          "version": "0.1.0",
          "dependencies": [],
          "api": {}
        }
        "#;

        let result = ten_validate_manifest_json_string(manifest);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_graph_invalid_additional_property() {
        let property = r#"
        {
          "_ten": {
            "predefined_graphs": [
              {
                "name": "default",
                "nodes": [
                  {
                    "type": "extension",
                    "name": "default_extension_cpp",
                    "addon": "default_extension_cpp",
                    "extension_group": "default_extension_group",
                    "should_not_present": "aa"
                  }
                ]
              }
            ]
          }
        }
        "#;

        let result = ten_validate_property_json_string(property);
        assert!(result.is_err());

        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("Additional properties are not allowed"));
    }

    #[test]
    fn test_validate_invalid_command_name_empty() {
        let property = r#"
        {
          "_ten": {
            "predefined_graphs": [
              {
                "name": "default",
                "nodes": [{
                  "type": "extension",
                  "name": "default_extension_cpp",
                  "addon": "default_extension_cpp",
                  "extension_group": "default_extension_group"
                }],
                "connections": [
                  {
                    "extension": "default_extension_cpp",
                    "cmd": [
                      {
                        "name": "",
                        "dest": [
                          {
                            "extension": "default_extension_cpp"
                          }
                        ]
                      }
                    ]
                  }
                ]
              }
            ]
          }
        }
        "#;

        let result = ten_validate_property_json_string(property);
        assert!(result.is_err());

        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("is shorter than 1 character"));
    }

    #[test]
    fn test_validate_invalid_cmd_no_dest() {
        let property = r#"
        {
          "_ten": {
            "predefined_graphs": [
              {
                "name": "default",
                "nodes": [{
                  "type": "extension",
                  "name": "default_extension_cpp",
                  "addon": "default_extension_cpp",
                  "extension_group": "default_extension_group"
                }],
                "connections": [
                  {
                    "extension": "default_extension_cpp",
                    "cmd": [
                      {
                        "name": "demo",
                        "dest": []
                      }
                    ]
                  }
                ]
              }
            ]
          }
        }
        "#;

        let result = ten_validate_property_json_string(property);
        assert!(result.is_err());

        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("[] has less than 1 item"));
    }

    #[test]
    fn test_validate_extension_property() {
        let property = r#"
        {
          "_ten": {
            "predefined_graphs": [
              {
                "name": "default",
                "nodes": [
                  {
                    "type": "extension",
                    "name": "default_extension_cpp",
                    "addon": "default_extension_cpp",
                    "extension_group": "default_extension_group",
                    "property": {
                      "a": 1
                    }
                  }
                ]
              }
            ]
          }
        }
        "#;

        let result = ten_validate_property_json_string(property);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_extension_no_cmds() {
        let property = r#"
        {
          "_ten": {
            "predefined_graphs": [
              {
                "name": "default",
                "nodes": [
                  {
                    "type": "extension",
                    "name": "default_extension_cpp",
                    "addon": "default_extension_cpp",
                    "extension_group": "default_extension_group"
                  }
                ]
              }
            ]
          }
        }
        "#;

        let result = ten_validate_property_json_string(property);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_api_cmd_in_success_1() {
        let manifest = r#"
        {
          "type": "app",
          "name": "default_app_cpp",
          "version": "0.1.0",
          "dependencies": [],
          "api": {
            "cmd_in": [
              {
                "name": "foo",
                "property": {
                  "a": {
                    "type": "int8"
                  },
                  "b": {
                    "type": "uint8"
                  },
                  "c": {
                    "type": "array",
                    "items": {
                      "type": "string"
                    }
                  },
                  "d": {
                    "type": "object",
                    "properties": {
                      "e": {
                        "type": "float32"
                      }
                    }
                  }
                },
                "result": {
                  "property": {
                    "a": {
                      "type": "buf"
                    },
                    "detail": {
                      "type": "buf"
                    }
                  }
                }
              }
            ]
          }
        }
        "#;

        let result = ten_validate_manifest_json_string(manifest);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_api_cmd_in_success_2() {
        let manifest = r#"
        {
          "type": "app",
          "name": "default_app_cpp",
          "version": "0.1.0",
          "dependencies": [],
          "api": {
            "cmd_in": [
              {
                "name": "foo",
                "property": {
                  "a": {
                    "type": "int8"
                  },
                  "b": {
                    "type": "uint8"
                  },
                  "c": {
                    "type": "array",
                    "items": {
                      "type": "string"
                    }
                  },
                  "d": {
                    "type": "object",
                    "properties": {
                      "e": {
                        "type": "float32"
                      }
                    }
                  }
                },
                "required": ["a","b"],
                "result": {
                  "property": {
                    "a": {
                      "type": "buf"
                    },
                    "detail": {
                      "type": "buf"
                    }
                  }
                }
              }
            ]
          }
        }
        "#;

        let result = ten_validate_manifest_json_string(manifest);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_api_cmd_in_success_3() {
        let manifest = r#"
        {
          "type": "app",
          "name": "default_app_cpp",
          "version": "0.1.0",
          "dependencies": [],
          "api": {
            "cmd_in": [
              {
                "name": "foo",
                "property": {
                  "a": {
                    "type": "int8"
                  },
                  "b": {
                    "type": "uint8"
                  },
                  "c": {
                    "type": "array",
                    "items": {
                      "type": "string"
                    }
                  },
                  "d": {
                    "type": "object",
                    "properties": {
                      "e": {
                        "type": "float32"
                      }
                    }
                  }
                },
                "required": ["a","b"],
                "result": {
                  "property": {
                    "a": {
                      "type": "buf"
                    },
                    "detail": {
                      "type": "buf"
                    }
                  },
                  "required": ["a"]
                }
              }
            ]
          }
        }
        "#;

        let result = ten_validate_manifest_json_string(manifest);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_api_cmd_in_has_nested_object_required() {
        let manifest = r#"
        {
          "type": "app",
          "name": "default_app_cpp",
          "version": "0.1.0",
          "dependencies": [],
          "api": {
            "cmd_in": [
              {
                "name": "foo",
                "property": {
                  "a": {
                    "type": "int8"
                  },
                  "b": {
                    "type": "uint8"
                  },
                  "c": {
                    "type": "array",
                    "items": {
                      "type": "string"
                    }
                  },
                  "d": {
                    "type": "object",
                    "properties": {
                      "e": {
                        "type": "float32"
                      },
                      "f": {
                        "type": "string"
                      }
                    },
                    "required": ["e"]
                  }
                },
                "required": ["a","b"],
                "result": {
                  "property": {
                    "a": {
                      "type": "buf"
                    },
                    "detail": {
                      "type": "buf"
                    }
                  }
                }
              }
            ]
          }
        }
        "#;

        let result = ten_validate_manifest_json_string(manifest);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_api_cmd_in_fail_1() {
        let manifest = r#"
        {
          "type": "app",
          "name": "default_app_cpp",
          "version": "0.1.0",
          "dependencies": [],
          "api": {
            "cmd_in": [
              {
                "name": "foo",
                "property": {
                  "//a": {
                    "type": "int8"
                  },
                  "b": {
                    "type": "uint8"
                  },
                  "c": {
                    "type": "array",
                    "items": {
                      "type": "string"
                    }
                  },
                  "d": {
                    "type": "object",
                    "properties": {
                      "e": {
                        "type": "float32"
                      }
                    }
                  }
                },
                "result": {
                  "property": {
                    "a": {
                      "type": "buf"
                    },
                    "detail": {
                      "type": "buf"
                    }
                  }
                }
              }
            ]
          }
        }
        "#;

        let result = ten_validate_manifest_json_string(manifest);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_api_cmd_in_fail_2() {
        let manifest = r#"
        {
          "type": "app",
          "name": "default_app_cpp",
          "version": "0.1.0",
          "dependencies": [],
          "api": {
            "cmd_in": [
              {
                "name": "foo",
                "property": {
                  "a": {
                    "type": "int8"
                  },
                  "b": {
                    "type": "uint8"
                  },
                  "c": {
                    "type": "array",
                    "items": {
                      "type": "string"
                    }
                  },
                  "d": {
                    "type": "object",
                    "properties": {
                      "//e": {
                        "type": "float32"
                      }
                    }
                  }
                },
                "result": {
                  "property": {
                    "a": {
                      "type": "buf"
                    },
                    "detail": {
                      "type": "buf"
                    }
                  }
                }
              }
            ]
          }
        }
        "#;

        let result = ten_validate_manifest_json_string(manifest);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_api_cmd_in_fail_3() {
        let manifest = r#"
        {
          "type": "app",
          "name": "default_app_cpp",
          "version": "0.1.0",
          "dependencies": [],
          "api": {
            "cmd_in": [
              {
                "name": "foo",
                "property": {
                  "a": {
                    "type": "int8"
                  },
                  "b": {
                    "type": "uint8"
                  },
                  "c": {
                    "type": "array",
                    "items": {
                      "type": "string"
                    }
                  },
                  "d": {
                    "type": "object",
                    "properties": {
                      "e": {
                        "type": "float32"
                      }
                    }
                  }
                },
                "result": {
                  "property": {
                    "//a": {
                      "type": "buf"
                    },
                    "detail": {
                      "type": "buf"
                    }
                  }
                }
              }
            ]
          }
        }
        "#;

        let result = ten_validate_manifest_json_string(manifest);
        assert!(result.is_err());
    }

    #[test]
    fn test_manifest_validate_api_cmd_in_result_must_have_prop() {
        let manifest = r#"
        {
          "type": "extension",
          "name": "default_extension_cpp",
          "version": "0.1.0",
          "dependencies": [],
          "api": {
            "cmd_in": [
              {
                "name": "foo",
                "property": {
                  "a": {
                    "type": "int8"
                  },
                  "b": {
                    "type": "uint8"
                  },
                  "c": {
                    "type": "array",
                    "items": {
                      "type": "string"
                    }
                  },
                  "d": {
                    "type": "object",
                    "properties": {
                      "e": {
                        "type": "float32"
                      }
                    }
                  }
                },
                "result": {
                }
              }
            ]
          }
        }
        "#;

        let result = ten_validate_manifest_json_string(manifest);
        assert!(result.is_err());
    }

    #[test]
    fn test_manifest_validate_api_cmd_in_has_additional_prop() {
        let manifest = r#"
        {
          "type": "extension",
          "name": "embedding",
          "version": "0.1.0",
  "dependencies": [
            {
              "type": "system",
              "name": "ten_runtime_python",
              "version": "0.2.2"
            }
          ],
          "api": {
            "property": {
              "api_key": {
                "type": "string"
              },
              "model": {
                "type": "string"
              }
            },
            "cmd_in": [
              {
                "name": "embed",
                "property": {
                  "input": {
                    "type": "string"
                  }
                },
                "status": {
                  "property": {
                    "output": {
                      "type": "string"
                    },
                    "code": {
                      "type": "string"
                    },
                    "message": {
                      "type": "string"
                    }
                  }
                }
              },
              {
                "name": "embed_batch",
                "property": {
                  "inputs": {
                    "type": "array",
                    "items": {
                      "type": "string"
                    }
                  }
                },
                "status": {
                  "property": {
                    "output": {
                      "type": "string"
                    },
                    "code": {
                      "type": "string"
                    },
                    "message": {
                      "type": "string"
                    }
                  }
                }
              }
            ],
            "cmd_out": []
          }
        }
        "#;

        let result = ten_validate_manifest_json_string(manifest);
        assert!(result.is_err());

        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("Additional properties are not allowed ('status' was unexpected) @ /api/cmd_in/0"));
    }

    #[test]
    fn test_validate_api_only_array_has_items() {
        let manifest = r#"
        {
          "type": "app",
          "name": "default_app_cpp",
          "version": "0.1.0",
          "dependencies": [],
          "api": {
            "cmd_in": [
              {
                "name": "foo",
                "property": {
                  "a": {
                    "type": "int8",
                    "items": {
                      "type": "string"
                    }
                  }
                }
              }
            ]
          }
        }
        "#;

        let result = ten_validate_manifest_json_string(manifest);
        assert!(result.is_err());

        assert!(result
            .unwrap_err()
            .to_string()
            .contains("{\"required\":[\"items\"]} is not allowed for"));
    }

    #[test]
    fn test_validate_api_only_object_has_properties() {
        let manifest = r#"
        {
          "type": "app",
          "name": "default_app_cpp",
          "version": "0.1.0",
          "dependencies": [],
          "api": {
            "cmd_in": [
              {
                "name": "foo",
                "property": {
                  "a": {
                    "type": "string",
                    "properties": {
                      "a": {
                        "type": "string"
                      }
                    }
                  }
                }
              }
            ]
          }
        }
        "#;

        let result = ten_validate_manifest_json_string(manifest);
        assert!(result.is_err());

        assert!(result
            .unwrap_err()
            .to_string()
            .contains("{\"required\":[\"properties\"]} is not allowed for"));
    }

    #[test]
    fn test_graph_msg_conversions() {
        let property = r#"
        {
          "_ten": {
            "predefined_graphs": [{
              "name": "default",
              "auto_start": false,
              "nodes": [{
                "type": "extension",
                "name": "test_extension_1",
                "addon": "result_mapping_1__test_extension_1",
                "extension_group": "result_mapping_1__extension_group"
              },{
                "type": "extension",
                "name": "test_extension_2",
                "addon": "result_mapping_1__test_extension_2",
                "extension_group": "result_mapping_1__extension_group"
              }],
              "connections": [{
                "app": "msgpack://127.0.0.1:8001/",
                "extension": "test_extension_1",
                "cmd": [{
                  "name": "hello_world",
                  "dest": [{
                    "app": "msgpack://127.0.0.1:8001/",
                    "extension": "test_extension_2",
                    "msg_conversion": {
                      "type": "per_property",
                      "rules": [{
                        "path": "_ten.name",
                        "conversion_mode": "fixed_value",
                        "value": "hello mapping"
                      },{
                        "path": "test_group.test_property_name",
                        "conversion_mode": "from_original",
                        "original_path": "test_property"
                      }],
                      "result": {
                        "type": "per_property",
                        "rules": [{
                          "path": "resp_group.resp_property_name",
                          "conversion_mode": "from_original",
                          "original_path": "resp_property"
                        }]
                      }
                    }
                  }]
                }]
              }]
            }]
          }
        }
        "#;

        let result = ten_validate_property_json_string(property);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_property_cmd_must_be_alphanumeric() {
        let property = r#"
        {
          "_ten": {
            "predefined_graphs": [
              {
                "name": "default",
                "nodes": [{
                  "type": "extension",
                  "name": "default_extension_cpp",
                  "addon": "default_extension_cpp",
                  "extension_group": "default_extension_group"
                }],
                "connections": [
                  {
                    "extension": "default_extension_cpp",
                    "cmd": [
                      {
                        "name": "invalid command",
                        "dest": [
                          {
                            "extension": "default_extension_cpp"
                          }
                        ]
                      }
                    ]
                  }
                ]
              }
            ]
          }
        }
        "#;

        let result = ten_validate_property_json_string(property);
        assert!(result.is_err());

        assert!(result.unwrap_err().to_string().contains("does not match"));
    }

    #[test]
    fn test_validate_manifest_interface_valid() {
        let manifest = r#"
        {
          "type": "extension",
          "name": "a",
          "version": "0.1.0",
                  "api": {
            "interface_in": [
              {
                "name": "ia",
                "$ref": "https://a.b.c/d.json"
              },
              {
                "name": "ib",
                "$ref": "file://c.json"
              }
            ],
            "interface_out": [
              {
                "name": "ic",
                "cmd": [
                  {
                    "name": "hello"
                  }
                ]
              }
            ]
          }
        }
        "#;

        let result = ten_validate_manifest_json_string(manifest);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_manifest_interface_invalid_ref() {
        let manifest = r#"
        {
          "type": "extension",
          "name": "a",
          "version": "0.1.0",
                  "api": {
            "interface_in": [
              {
                "name": "ia",
                "$ref": "ftp://a.b.c/d.json"
              }
            ]
          }
        }
        "#;

        let result = ten_validate_manifest_json_string(manifest);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains(
            "not valid under any of the schemas listed in the 'oneOf' keyword"
        ));
    }

    #[test]
    fn test_validate_property_json_valid() {
        let property = r#"
        {
          "_ten": {
            "log_level": 2,
            "log_file": "api.log"
          },
          "a": 1,
          "b": "2",
          "c": [1, 2, 3],
          "d": {
            "e": 1.0
          }
        }
        "#;

        let result = ten_validate_property_json_string(property);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_property_key_must_be_alphanumeric() {
        let property = r#"
        {
          "invalid-key": 1
        }
        "#;

        let result = ten_validate_property_json_string(property);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_manifest_lock_empty() {
        let manifest_lock = r#"{}"#;

        // The 'packages' field is required in the manifest lock file.
        let result = validate_manifest_lock_json_string(manifest_lock);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_manifest_lock_empty_packages() {
        let manifest_lock = r#"{
          "version": 1,
          "packages": []
        }"#;

        let result = validate_manifest_lock_json_string(manifest_lock);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_manifest_lock_invalid_lock_version() {
        let manifest_lock = r#"{
          "version": 0,
          "packages": []
        }"#;

        let result = validate_manifest_lock_json_string(manifest_lock);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_manifest_lock_duplicated_packages() {
        // NOTE: The 'type' combined with 'name' must be unique. But the json
        // schema check does not support this. The check should be done
        // in the code.
        let manifest_lock = r#"{
          "version": 1,
          "packages": [
            {
              "type": "extension",
              "name": "ext_a",
              "version": "1.0.0",
              "hash": "e8dc07a47927e9a650d23f77676b798e0856dd169fea70e7db57d57095261a68"
            },
            {
              "type": "extension",
              "name": "ext_a",
              "version": "1.0.0",
              "hash": "e8dc07a47927e9a650d23f77676b798e0856dd169fea70e7db57d57095261a68"
            }
          ]
        }"#;

        let result = validate_manifest_lock_json_string(manifest_lock);
        assert!(result.is_err());

        let err_reason = result.unwrap_err().to_string();
        assert!(err_reason.contains("has non-unique elements"));
    }

    #[test]
    fn test_validate_manifest_lock_missing_field_in_pkg() {
        // Miss hash field in the package.
        let manifest_lock = r#"{
          "version": 1,
          "packages": [
            {
              "type": "extension",
              "name": "ext_a",
              "version": "1.0.0"
            }
          ]
        }"#;

        let result = validate_manifest_lock_json_string(manifest_lock);
        assert!(result.is_err());

        let err_reason = result.unwrap_err().to_string();
        assert!(err_reason.contains("is a required property"));
    }

    #[test]
    fn test_validate_manifest_lock_invalid_version() {
        // The version field must be a fixed version but not a range.
        let manifest_lock = r#"{
          "version": 1,
          "packages": [
            {
              "type": "extension",
              "name": "ext_a",
              "version": ">1.0.0",
              "hash": "e8dc07a47927e9a650d23f77676b798e0856dd169fea70e7db57d57095261a68"
            }
          ]
        }"#;

        let result = validate_manifest_lock_json_string(manifest_lock);
        assert!(result.is_err());

        let err_reason = result.unwrap_err().to_string();
        assert!(err_reason.contains("does not match"));
    }

    #[test]
    fn test_validate_manifest_lock_empty_dependency() {
        let manifest_lock = r#"{
          "version": 1,
          "packages": [
            {
              "type": "extension",
              "name": "ext_a",
              "version": "1.0.0",
              "hash": "e8dc07a47927e9a650d23f77676b798e0856dd169fea70e7db57d57095261a68",
      "dependencies": []
            }
          ]
        }"#;

        let result = validate_manifest_lock_json_string(manifest_lock);
        assert!(result.is_err());

        let err_reason = result.unwrap_err().to_string();
        assert!(err_reason.contains("has less than 1 item"));
    }

    #[test]
    fn test_validate_manifest_lock_invalid_supports() {
        let manifest_lock = r#"{
          "version": 1,
          "packages": [
            {
              "type": "extension",
              "name": "ext_a",
              "version": "1.0.0",
              "hash": "e8dc07a47927e9a650d23f77676b798e0856dd169fea70e7db57d57095261a68",
              "supports": [
                {
                  "os": "linux",
                  "arch": "x63"
                }
              ]
            }
          ]
        }"#;

        let result = validate_manifest_lock_json_string(manifest_lock);
        assert!(result.is_err());

        let err_reason = result.unwrap_err().to_string();
        assert!(err_reason.contains("is not one of"));
    }

    #[test]
    fn test_validate_manifest_lock_duplicated_dependencies() {
        let manifest_lock = r#"{
          "version": 1,
          "packages": [
            {
              "type": "extension",
              "name": "ext_a",
              "version": "1.0.0",
              "hash": "e8dc07a47927e9a650d23f77676b798e0856dd169fea70e7db57d57095261a68",
      "dependencies": [
                {
                  "type": "extension",
                  "name": "ext_a"
                },
                {
                  "type": "extension",
                  "name": "ext_a"
                }
              ]
            }
          ]
        }"#;

        let result = validate_manifest_lock_json_string(manifest_lock);
        assert!(result.is_err());

        let err_reason = result.unwrap_err().to_string();
        assert!(err_reason.contains("has non-unique elements"));
    }

    #[test]
    fn test_validate_manifest_lock_success() {
        let manifest_lock = r#"{
        "version": 1,
        "packages": [
          {
            "type": "extension",
            "name": "ext_b",
            "version": "1.0.0",
            "hash": "e6d7ff0aefd1a0618c9f7d8c154b1b90c917390bd00d2a107eae616a36a99391",
    "dependencies": [
              {
                "type": "extension",
                "name": "ext_a"
              }
            ]
          },
          {
            "type": "extension",
            "name": "ext_1",
            "version": "2.0.0",
            "hash": "e64b5c3c66a3014f2c58299d663533eb984ac9e71aa50067f6f3c3b9d409ccdf",
    "dependencies": [
              {
                "type": "extension",
                "name": "ext_3"
              }
            ]
          },
          {
            "type": "extension",
            "name": "ext_3",
            "version": "1.0.0",
            "hash": "2c61c0fc8b3cd2da7457a779b2db423ae93c3b6cae7347cb979453f1ac17ccc0"
          },
          {
            "type": "extension",
            "name": "ext_a",
            "version": "1.2.2",
            "hash": "6fee978cd201b108211b52323a078c5222fd0bc545468b5989d7e42d0f5b7395"
          }
        ]
      }"#;

        let result = validate_manifest_lock_json_string(manifest_lock);
        assert!(result.is_ok());
    }
}
