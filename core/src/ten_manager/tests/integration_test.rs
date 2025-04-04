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

    mod cmd {
        mod check {
            mod check_graph;
        }
        mod modify {
            mod jq_util;
        }
    }

    mod designer {
        mod builtin_function_install;
        mod builtin_function_install_all;
        mod get_apps;
        mod get_graphs;
        mod get_packages_scripts;
        mod get_registry_packages;
        mod load_apps;
        mod reload_apps;
        mod terminal;
    }

    mod graph {
        mod node {
            pub mod remove_all_connections;
            pub mod remove_connection_dest;
            pub mod update_and_preserve_order;
        }
        mod connection {
            pub mod update_and_preserve_order;
        }
    }

    // New test modules
    mod version_utils;

    mod schema;

    mod pkg_info {
        mod manifest {
            mod to_file;
        }
    }

    mod registry {
        mod local;
    }
}
