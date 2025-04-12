//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::collections::HashMap;

use anyhow::{Context, Ok, Result};

use crate::pkg_info::manifest::{
    api::{
        ManifestApi, ManifestApiCmdLike, ManifestApiDataLike,
        ManifestPropertyAttributes,
    },
    Manifest,
};

use super::runtime_interface::{create_schema_from_json, TenSchema};

/// Represents the schema for a command and its result.
///
/// This structure holds two optional schemas:
/// - `cmd`: The schema that defines the structure of the command.
/// - `result`: The schema that defines the structure of the command's result.
#[derive(Debug, Clone, Default)]
pub struct CmdSchema {
    /// Schema for the command structure.
    pub cmd: Option<TenSchema>,
    /// Schema for the command's result structure.
    pub result: Option<TenSchema>,
}

#[derive(Debug, Clone, Default)]
pub struct SchemaStore {
    /// Schema for property definitions.
    pub property: Option<TenSchema>,

    /// Command schemas for incoming commands.
    pub cmd_in: HashMap<String, CmdSchema>,
    /// Command schemas for outgoing commands.
    pub cmd_out: HashMap<String, CmdSchema>,

    /// Data schemas for incoming data.
    pub data_in: HashMap<String, TenSchema>,
    /// Data schemas for outgoing data.
    pub data_out: HashMap<String, TenSchema>,

    /// Schema for incoming video frames.
    pub video_frame_in: HashMap<String, TenSchema>,
    /// Schema for outgoing video frames.
    pub video_frame_out: HashMap<String, TenSchema>,

    /// Schema for incoming audio frames.
    pub audio_frame_in: HashMap<String, TenSchema>,
    /// Schema for outgoing audio frames.
    pub audio_frame_out: HashMap<String, TenSchema>,
}

impl SchemaStore {
    /// Creates a SchemaStore from a Manifest.
    ///
    /// This function extracts API schemas from the manifest and constructs a
    /// SchemaStore containing all the command, data, and frame schemas
    /// defined in the manifest.
    pub fn from_manifest(manifest: &Manifest) -> Result<Option<Self>> {
        if let Some(api) = &manifest.api {
            let mut schema_store = SchemaStore::default();
            schema_store.parse_schemas_from_manifest(api)?;
            Ok(Some(schema_store))
        } else {
            Ok(None)
        }
    }

    /// Parses schemas from a ManifestApi and populates the SchemaStore.
    ///
    /// This method processes all schema types defined in the manifest API:
    /// - Property schemas
    /// - Command schemas (both incoming and outgoing)
    /// - Data schemas (both incoming and outgoing)
    /// - Video frame schemas (both incoming and outgoing)
    /// - Audio frame schemas (both incoming and outgoing)
    ///
    /// Each schema type is parsed and stored in the appropriate collection
    /// within the SchemaStore.
    pub fn parse_schemas_from_manifest(
        &mut self,
        manifest_api: &ManifestApi,
    ) -> Result<()> {
        // Parse property schema if defined.
        if let Some(property) = &manifest_api.property {
            let mut property_schema_value: serde_json::Value =
                serde_json::json!({"type": "object"});
            let property_schema_object =
                property_schema_value.as_object_mut().unwrap();
            property_schema_object.insert(
                "properties".to_string(),
                serde_json::to_value(property)?,
            );

            let schema = create_schema_from_json(
                serde_json::to_value(property_schema_object)
                    .as_ref()
                    .unwrap(),
            )?;
            self.property = Some(schema);
        }

        // Parse incoming command schemas.
        if let Some(cmd_in_schema) = &manifest_api.cmd_in {
            parse_cmds_schema_from_manifest(cmd_in_schema, &mut self.cmd_in)
                .with_context(|| "Failed to parse cmd_in schema")?;
        }

        // Parse outgoing command schemas.
        if let Some(cmd_out_schema) = &manifest_api.cmd_out {
            parse_cmds_schema_from_manifest(cmd_out_schema, &mut self.cmd_out)
                .with_context(|| "Failed to parse cmd_out schema")?;
        }

        // Parse incoming data schemas.
        if let Some(data_in_schema) = &manifest_api.data_in {
            parse_msg_schema_from_manifest(data_in_schema, &mut self.data_in)
                .with_context(|| "Failed to parse data_in schema")?;
        }

        // Parse outgoing data schemas.
        if let Some(data_out_schema) = &manifest_api.data_out {
            parse_msg_schema_from_manifest(data_out_schema, &mut self.data_out)
                .with_context(|| "Failed to parse data_out schema")?;
        }

        // Parse incoming video frame schemas.
        if let Some(video_frame_in_schema) = &manifest_api.video_frame_in {
            parse_msg_schema_from_manifest(
                video_frame_in_schema,
                &mut self.video_frame_in,
            )
            .with_context(|| "Failed to parse video_frame_in schema")?;
        }

        // Parse outgoing video frame schemas.
        if let Some(video_frame_out_schema) = &manifest_api.video_frame_out {
            parse_msg_schema_from_manifest(
                video_frame_out_schema,
                &mut self.video_frame_out,
            )
            .with_context(|| "Failed to parse video_frame_out schema")?;
        }

        // Parse incoming audio frame schemas.
        if let Some(audio_frame_in_schema) = &manifest_api.audio_frame_in {
            parse_msg_schema_from_manifest(
                audio_frame_in_schema,
                &mut self.audio_frame_in,
            )
            .with_context(|| "Failed to parse audio_frame_in schema")?;
        }

        // Parse outgoing audio frame schemas.
        if let Some(audio_frame_out_schema) = &manifest_api.audio_frame_out {
            parse_msg_schema_from_manifest(
                audio_frame_out_schema,
                &mut self.audio_frame_out,
            )
            .with_context(|| "Failed to parse audio_frame_out schema")?;
        }

        Ok(())
    }
}

fn parse_cmds_schema_from_manifest(
    manifest_cmds: &Vec<ManifestApiCmdLike>,
    target_map: &mut HashMap<String, CmdSchema>,
) -> Result<()> {
    for manifest_cmd in manifest_cmds {
        let cmd_name = manifest_cmd.name.clone();
        let cmd_schema = create_cmd_schema_from_manifest(manifest_cmd)?;
        if let Some(schema) = cmd_schema {
            let present = target_map.insert(cmd_name.clone(), schema);
            if present.is_some() {
                return Err(anyhow::anyhow!(
                    "duplicated schema definition for cmd {}.",
                    cmd_name
                ));
            }
        }
    }

    Ok(())
}

fn parse_msg_schema_from_manifest(
    manifest_data: &Vec<ManifestApiDataLike>,
    target_map: &mut HashMap<String, TenSchema>,
) -> Result<()> {
    for manifest_data in manifest_data {
        let data_name = manifest_data.name.clone();
        let schema = create_msg_schema_from_manifest(manifest_data)?;
        if let Some(schema) = schema {
            let present = target_map.insert(data_name.clone(), schema);
            if present.is_some() {
                return Err(anyhow::anyhow!(
                    "duplicated schema definition for msg {}.",
                    data_name
                ));
            }
        }
    }

    Ok(())
}

// The TEN schema system will assign a type to each schema type. For example,
// the type for an object schema is 'object,' and the type for an array schema
// is 'array,' and so on. If the schema is of object type, it also needs to have
// a top-level field called 'properties' to hold the contents of the object
// schema. However, the content of the JSON will be somewhat simplified to avoid
// the need for manually writing redundant information. Therefore, the parsing
// logic will need to dynamically add some appropriate fields on the fly to meet
// the requirements of the TEN schema format.
//
// The 'property' field is of object type, so its schema definition needs to be
// an object with a 'type' field equals to 'object'. The expected schema json of
// 'property' field is:
//
// {
//   "type": "object",
//   "properties": {
//
//   },
//   "required": []
// }
fn create_property_schema(
    property: &Option<HashMap<String, ManifestPropertyAttributes>>,
    required: &Option<Vec<String>>,
) -> Result<TenSchema> {
    let mut property_json_value = serde_json::json!({});
    let property_json_object = property_json_value.as_object_mut().unwrap();

    if let Some(prop_map) = property {
        prop_map.iter().for_each(|(key, attr)| {
            property_json_object
                .insert(key.clone(), serde_json::to_value(attr).unwrap());
        });
    }

    let mut property_schema_value: serde_json::Value =
        serde_json::json!({"type": "object"});
    let property_schema_object = property_schema_value.as_object_mut().unwrap();
    property_schema_object.insert(
        "properties".to_string(),
        serde_json::to_value(property_json_object)?,
    );
    if let Some(required) = required {
        property_schema_object
            .insert("required".to_string(), serde_json::to_value(required)?);
    }

    create_schema_from_json(
        serde_json::to_value(property_schema_object)
            .as_ref()
            .unwrap(),
    )
}

fn create_cmd_schema_from_manifest(
    manifest_cmd: &ManifestApiCmdLike,
) -> Result<Option<CmdSchema>> {
    let mut schema = CmdSchema::default();
    if let Some(manifest_result) = &manifest_cmd.result {
        let result_schema = create_property_schema(
            &manifest_result.property,
            &manifest_result.required,
        )?;
        schema.result = Some(result_schema);
    }

    if let Some(manifest_property) = &manifest_cmd.property {
        let property_schema = create_property_schema(
            &Some(manifest_property.clone()),
            &manifest_cmd.required,
        )?;
        schema.cmd = Some(property_schema);
    }

    if schema.cmd.is_none() && schema.result.is_none() {
        Ok(None)
    } else {
        Ok(Some(schema))
    }
}

fn create_msg_schema_from_manifest(
    manifest_data: &ManifestApiDataLike,
) -> Result<Option<TenSchema>> {
    if let Some(manifest_property) = &manifest_data.property {
        let schema = create_property_schema(
            &Some(manifest_property.clone()),
            &manifest_data.required,
        )?;
        Ok(Some(schema))
    } else {
        Ok(None)
    }
}

pub fn are_ten_schemas_compatible(
    source: Option<&TenSchema>,
    target: Option<&TenSchema>,
) -> Result<()> {
    if source.is_none() && target.is_none() {
        return Ok(());
    }

    if source.is_none() || target.is_none() {
        return Err(anyhow::anyhow!("source or target schema is undefined."));
    }

    let source = source.unwrap();
    let target = target.unwrap();
    source.is_compatible_with(target)
}

pub fn are_cmd_schemas_compatible(
    source: Option<&CmdSchema>,
    target: Option<&CmdSchema>,
) -> Result<()> {
    if source.is_none() && target.is_none() {
        return Ok(());
    }

    // TODO(Liu): The compatibility check should be more strict, ex:
    // * If the source is none, but the target has 'required' keyword, then it
    //   is not compatible.
    // * If the target is none, then it is compatible.
    //
    // But we do not have enough information in TenSchema, and it's not a good
    // idea to keep keywords info in Rust.
    if source.is_none() || target.is_none() {
        return Err(anyhow::anyhow!("source or target schema is undefined."));
    }

    let source = source.unwrap();
    let target = target.unwrap();
    are_ten_schemas_compatible(source.cmd.as_ref(), target.cmd.as_ref())?;
    are_ten_schemas_compatible(source.result.as_ref(), target.result.as_ref())?;

    Ok(())
}
