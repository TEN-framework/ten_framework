//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::sync::{Arc, Mutex};

/// Abstract all log output methods: CLI, WebSocket, etc.
trait TmanOutputTrait: Send + Sync {
    /// General information.
    fn output_line(&self, text: &str);
    fn output(&self, text: &str);

    /// Error information.
    fn output_err_line(&self, text: &str);

    /// Whether it is interactive (e.g., can block waiting for user input in CLI
    /// environment).
    fn is_interactive(&self) -> bool;
}

/// Output for CLI: directly println / eprintln.
pub struct TmanOutputCli;

impl TmanOutputTrait for TmanOutputCli {
    fn output_line(&self, text: &str) {
        println!("{}", text);
    }
    fn output(&self, text: &str) {
        print!("{}", text);
    }
    fn output_err_line(&self, text: &str) {
        eprintln!("{}", text);
    }
    fn is_interactive(&self) -> bool {
        true
    }
}

#[derive(Clone)]
pub struct TmanOutputWs {
    inner: Arc<Mutex<Vec<String>>>,
}

impl TmanOutputWs {
    pub fn new() -> Self {
        TmanOutputWs {
            inner: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Allow the upper layer to send accumulated outputs to the WebSocket
    /// client at the appropriate time.
    pub fn take_messages(&self) -> Vec<String> {
        let mut guard = self.inner.lock().unwrap();
        let msgs = guard.drain(..).collect();
        msgs
    }
}

impl Default for TmanOutputWs {
    fn default() -> Self {
        Self::new()
    }
}

impl TmanOutputTrait for TmanOutputWs {
    fn output_line(&self, text: &str) {
        let mut guard = self.inner.lock().unwrap();
        guard.push(format!("INFO: {}", text));
    }
    fn output(&self, text: &str) {
        let mut guard = self.inner.lock().unwrap();
        guard.push(format!("INFO: {}", text));
    }
    fn output_err_line(&self, text: &str) {
        let mut guard = self.inner.lock().unwrap();
        guard.push(format!("ERROR: {}", text));
    }
    fn is_interactive(&self) -> bool {
        false
    }
}

pub enum TmanOutput {
    Cli(TmanOutputCli),
    Ws(TmanOutputWs),
}

impl TmanOutput {
    pub fn output_line(&self, text: &str) {
        match self {
            Self::Cli(out) => out.output_line(text),
            Self::Ws(out) => out.output_line(text),
        }
    }

    pub fn output(&self, text: &str) {
        match self {
            Self::Cli(out) => out.output(text),
            Self::Ws(out) => out.output(text),
        }
    }

    pub fn output_err_line(&self, text: &str) {
        match self {
            Self::Cli(out) => out.output_err_line(text),
            Self::Ws(out) => out.output_err_line(text),
        }
    }

    pub fn is_interactive(&self) -> bool {
        match self {
            Self::Cli(out) => out.is_interactive(),
            Self::Ws(out) => out.is_interactive(),
        }
    }
}

impl Clone for TmanOutput {
    fn clone(&self) -> Self {
        match self {
            Self::Cli(_) => Self::Cli(TmanOutputCli),
            Self::Ws(out) => Self::Ws(TmanOutputWs {
                inner: Arc::clone(&out.inner),
            }),
        }
    }
}
