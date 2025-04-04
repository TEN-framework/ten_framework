//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//

// Each file in the tests directory is a separate crate, and will be compiled as
// an executable. We need a all-in-one executable to run all the tests, and we
// will copy the all-in-one executable to the out directory. We use this main
// file to achieve this purpose.

fn main() {
    println!("Running integration tests of tman...");
}

// Those following mods will be compiled in one executable.
mod test_case {
    mod common;

    mod cmd_check_graph;
    mod designer_builtin_function_install;
    mod designer_builtin_function_install_all;
    mod designer_get_apps;
    mod designer_get_graphs;
    mod designer_get_packages_scripts;
    mod designer_get_registry_packages;
    mod designer_load_apps;
    mod designer_reload_apps;
    mod designer_terminal;

    mod graph;
}
