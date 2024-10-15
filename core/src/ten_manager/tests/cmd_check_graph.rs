//
// Copyright © 2024 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use ten_manager::{
    cmd::cmd_check::cmd_check_graph::CheckGraphCommand, config::TmanConfig,
};

#[actix_rt::test]
async fn test_cmd_check_predefined_graph_success() {
    let tman_config = TmanConfig::default();
    let command = CheckGraphCommand {
        app: vec![
            "tests/test_data/cmd_check_predefined_graph_success".to_string()
        ],
        graph: None,
        predefined_graph_name: None,
    };

    let result = ten_manager::cmd::cmd_check::cmd_check_graph::execute_cmd(
        &tman_config,
        command,
    )
    .await;
    assert!(result.is_ok());
}

#[actix_rt::test]
async fn test_cmd_check_start_graph_multi_apps() {
    let tman_config = TmanConfig::default();
    let command = CheckGraphCommand {
        app: vec![
            "tests/test_data/cmd_check_start_graph_multi_apps/app_1"
                .to_string(),
            "tests/test_data/cmd_check_start_graph_multi_apps/app_2"
                .to_string(),
        ],
        graph: Some(
            include_str!(
                "test_data/cmd_check_start_graph_multi_apps/start_graph.json"
            )
            .to_string(),
        ),
        predefined_graph_name: None,
    };

    let result = ten_manager::cmd::cmd_check::cmd_check_graph::execute_cmd(
        &tman_config,
        command,
    )
    .await;

    // The output command of the source extension does not have a defined
    // schema, but the input command of the destination extension does. This
    // means that what the destination extension expects may not be provided by
    // the source extension, resulting in an error.
    assert!(result.is_err());
    eprintln!("{:?}", result);
}

#[actix_rt::test]
async fn test_cmd_check_app_in_graph_cannot_be_localhost() {
    let tman_config = TmanConfig::default();
    let command = CheckGraphCommand {
        app: vec!["tests/test_data/cmd_check_app_in_graph_cannot_be_localhost"
            .to_string()],
        graph: None,
        predefined_graph_name: None,
    };

    let result = ten_manager::cmd::cmd_check::cmd_check_graph::execute_cmd(
        &tman_config,
        command,
    )
    .await;

    assert!(result.is_err());
    eprintln!("{:?}", result);

    let msg = result.err().unwrap().to_string();
    assert!(msg
        .contains("the app uri should be some string other than 'localhost'"));
}

#[actix_rt::test]
async fn test_cmd_check_predefined_graph_only_check_specified() {
    let tman_config = TmanConfig::default();
    let command = CheckGraphCommand {
        app: vec![
            "tests/test_data/cmd_check_predefined_graph_only_check_specified"
                .to_string(),
        ],
        graph: None,
        predefined_graph_name: Some("default".to_string()),
    };

    let result = ten_manager::cmd::cmd_check::cmd_check_graph::execute_cmd(
        &tman_config,
        command,
    )
    .await;

    assert!(result.is_ok());
}

#[actix_rt::test]
async fn test_cmd_check_predefined_graph_check_all() {
    let tman_config = TmanConfig::default();
    let command = CheckGraphCommand {
        app: vec![
            "tests/test_data/cmd_check_predefined_graph_only_check_specified"
                .to_string(),
        ],
        graph: None,
        predefined_graph_name: None,
    };

    let result = ten_manager::cmd::cmd_check::cmd_check_graph::execute_cmd(
        &tman_config,
        command,
    )
    .await;

    // An extension name appears in the connections that is not defined in the
    // nodes.
    assert!(result.is_err());
    eprintln!("{:?}", result);

    let msg = result.err().unwrap().to_string();
    assert!(msg.contains("1/2 graphs failed"));
}
