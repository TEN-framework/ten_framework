//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use actix::Addr;
use serde::{Deserialize, Serialize};

use crate::output::TmanOutput;

use super::{BuiltinFunctionOutput, WsBuiltinFunction};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum OutboundMsg {
    #[serde(rename = "normal_output")]
    NormalOutput { data: String },

    #[serde(rename = "error_output")]
    ErrorOutput { data: String },

    #[serde(rename = "exit")]
    Exit { code: i32 },

    #[serde(rename = "error")]
    Error { msg: String },
}

pub struct TmanOutputWs {
    pub addr: Addr<WsBuiltinFunction>,
}

impl TmanOutput for TmanOutputWs {
    fn output_line(&self, text: &str) {
        self.addr
            .do_send(BuiltinFunctionOutput::NormalOutput(text.to_string()));
    }
    fn output(&self, text: &str) {
        self.addr
            .do_send(BuiltinFunctionOutput::NormalOutput(text.to_string()));
    }
    fn output_err_line(&self, text: &str) {
        self.addr
            .do_send(BuiltinFunctionOutput::ErrorOutput(text.to_string()));
    }
    fn is_interactive(&self) -> bool {
        false
    }
}
