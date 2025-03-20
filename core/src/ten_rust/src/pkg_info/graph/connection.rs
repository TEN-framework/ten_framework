//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::pkg_info::{graph::is_app_default_loc_or_none, localhost};

use super::{
    constants, msg_conversion::MsgAndResultConversion, GraphNodeAppDeclaration,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GraphConnection {
    #[serde(skip_serializing_if = "is_app_default_loc_or_none")]
    pub app: Option<String>,

    pub extension: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmd: Option<Vec<GraphMessageFlow>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Vec<GraphMessageFlow>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_frame: Option<Vec<GraphMessageFlow>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_frame: Option<Vec<GraphMessageFlow>>,
}

impl GraphConnection {
    pub fn validate_and_complete(
        &mut self,
        graph_node_app_declaration: &GraphNodeAppDeclaration,
    ) -> Result<()> {
        if let Some(app) = &self.app {
            if app.as_str() == localhost() {
                let err_msg =
                    if graph_node_app_declaration.is_single_app_graph() {
                        constants::ERR_MSG_APP_LOCALHOST_DISALLOWED_SINGLE
                    } else {
                        constants::ERR_MSG_APP_LOCALHOST_DISALLOWED_MULTI
                    };

                return Err(anyhow::anyhow!(err_msg));
            }

            if *graph_node_app_declaration
                == GraphNodeAppDeclaration::NoneDeclared
            {
                return Err(anyhow::anyhow!(
                    constants::ERR_MSG_APP_SHOULD_NOT_DECLARED
                ));
            }
        } else {
            if *graph_node_app_declaration
                != GraphNodeAppDeclaration::NoneDeclared
            {
                return Err(anyhow::anyhow!(
                    constants::ERR_MSG_APP_SHOULD_DECLARED
                ));
            }

            self.app = Some(localhost().to_string());
        }

        if let Some(cmd) = &mut self.cmd {
            for (idx, cmd_flow) in cmd.iter_mut().enumerate() {
                cmd_flow
                    .validate_and_complete(graph_node_app_declaration)
                    .map_err(|e| anyhow::anyhow!("cmd[{}].{}", idx, e))?;
            }
        }

        if let Some(data) = &mut self.data {
            for (idx, data_flow) in data.iter_mut().enumerate() {
                data_flow
                    .validate_and_complete(graph_node_app_declaration)
                    .map_err(|e| anyhow::anyhow!("data[{}].{}", idx, e))?;
            }
        }

        if let Some(audio_frame) = &mut self.audio_frame {
            for (idx, audio_flow) in audio_frame.iter_mut().enumerate() {
                audio_flow
                    .validate_and_complete(graph_node_app_declaration)
                    .map_err(|e| {
                        anyhow::anyhow!("audio_frame[{}].{}", idx, e)
                    })?;
            }
        }

        if let Some(video_frame) = &mut self.video_frame {
            for (idx, video_flow) in video_frame.iter_mut().enumerate() {
                video_flow
                    .validate_and_complete(graph_node_app_declaration)
                    .map_err(|e| {
                        anyhow::anyhow!("video_frame[{}].{}", idx, e)
                    })?;
            }
        }

        Ok(())
    }

    pub fn get_app_uri(&self) -> &str {
        self.app.as_ref().unwrap().as_str()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GraphMessageFlow {
    pub name: String,
    pub dest: Vec<GraphDestination>,
}

impl GraphMessageFlow {
    fn validate_and_complete(
        &mut self,
        graph_node_app_declaration: &GraphNodeAppDeclaration,
    ) -> Result<()> {
        for (idx, dest) in &mut self.dest.iter_mut().enumerate() {
            dest.validate_and_complete(graph_node_app_declaration)
                .map_err(|e| anyhow::anyhow!("dest[{}]: {}", idx, e))?;
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GraphDestination {
    #[serde(skip_serializing_if = "is_app_default_loc_or_none")]
    pub app: Option<String>,

    pub extension: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_conversion: Option<MsgAndResultConversion>,
}

impl GraphDestination {
    fn validate_and_complete(
        &mut self,
        graph_node_app_declaration: &GraphNodeAppDeclaration,
    ) -> Result<()> {
        if let Some(app) = &self.app {
            if app.as_str() == localhost() {
                let err_msg =
                    if graph_node_app_declaration.is_single_app_graph() {
                        constants::ERR_MSG_APP_LOCALHOST_DISALLOWED_SINGLE
                    } else {
                        constants::ERR_MSG_APP_LOCALHOST_DISALLOWED_MULTI
                    };

                return Err(anyhow::anyhow!(err_msg));
            }

            if *graph_node_app_declaration
                == GraphNodeAppDeclaration::NoneDeclared
            {
                return Err(anyhow::anyhow!(
                    constants::ERR_MSG_APP_SHOULD_NOT_DECLARED
                ));
            }
        } else {
            if *graph_node_app_declaration
                != GraphNodeAppDeclaration::NoneDeclared
            {
                return Err(anyhow::anyhow!(
                    constants::ERR_MSG_APP_SHOULD_DECLARED
                ));
            }

            self.app = Some(localhost().to_string());
        }

        if let Some(msg_conversion) = &self.msg_conversion {
            msg_conversion.validate().map_err(|e| {
                anyhow::anyhow!("invalid msg_conversion: {}", e)
            })?;
        }

        Ok(())
    }

    pub fn get_app_uri(&self) -> &str {
        self.app.as_ref().unwrap().as_str()
    }
}
