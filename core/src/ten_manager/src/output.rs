//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//

/// Abstract all log output methods: CLI, WebSocket, etc.
pub trait TmanOutput: Send + Sync {
    /// General information.
    fn normal_line(&self, text: &str);
    fn normal_partial(&self, text: &str);

    /// Error information.
    fn error_line(&self, text: &str);
    fn error_partial(&self, text: &str);

    /// Whether it is interactive (e.g., can block waiting for user input in CLI
    /// environment).
    fn is_interactive(&self) -> bool;
}

/// Output for CLI: directly println / eprintln.
pub struct TmanOutputCli;

impl TmanOutput for TmanOutputCli {
    fn normal_line(&self, text: &str) {
        println!("{}", text);
    }
    fn normal_partial(&self, text: &str) {
        print!("{}", text);
    }
    fn error_line(&self, text: &str) {
        eprintln!("{}", text);
    }
    fn error_partial(&self, text: &str) {
        eprint!("{}", text);
    }
    fn is_interactive(&self) -> bool {
        true
    }
}
