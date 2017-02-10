extern crate delivery;

use delivery::delivery_config::{DeliveryConfigV1, DeliveryConfig};
use std::collections::HashMap;
use support::paths::fixture_file;

fn setup() { }

mod v1 {
    use super::*;
    test!(load_config {
        let source_dir = fixture_file("test_v1_repo");
        let c_v1 = DeliveryConfigV1::load_config(&source_dir).unwrap();
        let mut build_nodes = HashMap::new();
        build_nodes.insert(
            "default".to_string(), vec!["name:delivery-builder*".to_string()]
        );

        assert_eq!(c_v1.version, "1".to_string());
        assert_eq!(c_v1.build_cookbook, "./.delivery/build_cookbook".to_string());
        assert_eq!(c_v1.skip_phases, None);
        assert_eq!(c_v1.build_nodes, Some(build_nodes));
    });
}

mod v2 {
    use super::*;

    test!(load_config {
        let source_dir = fixture_file("test_repo");
        let c_v2 = DeliveryConfig::load_config(&source_dir).unwrap();
        let mut build_nodes = HashMap::new();
        build_nodes.insert(
            "default".to_string(), vec!["name:delivery-builder*".to_string()]
        );
        let mut build_cookbook = HashMap::new();
        build_cookbook.insert("name".to_string(),
                              "delivery_test".to_string());
        build_cookbook.insert("path".to_string(),
                              "cookbooks/delivery_test".to_string());

        assert_eq!(c_v2.version, "2".to_string());
        assert_eq!(c_v2.skip_phases, Some(Vec::new()));
        assert_eq!(c_v2.dependencies, Some(Vec::new()));
        assert_eq!(c_v2.build_cookbook, build_cookbook);
        assert_eq!(c_v2.build_nodes, Some(build_nodes));
        assert!(c_v2.job_dispatch.is_none());
    });

    test!(load_v1_config_and_return_v2 {
        let source_dir = fixture_file("test_v1_repo");
        let c_v1_to_v2 = DeliveryConfig::load_config(&source_dir).unwrap();
        let mut build_nodes = HashMap::new();
        build_nodes.insert(
            "default".to_string(), vec!["name:delivery-builder*".to_string()]
        );
        let mut build_cookbook = HashMap::new();
        build_cookbook.insert("name".to_string(),
                              "build_cookbook".to_string());
        build_cookbook.insert("path".to_string(),
                              "./.delivery/build_cookbook".to_string());

        assert_eq!(c_v1_to_v2.version, "1".to_string());
        assert_eq!(c_v1_to_v2.build_cookbook, build_cookbook);
        assert_eq!(c_v1_to_v2.build_nodes, Some(build_nodes));
        assert!(c_v1_to_v2.skip_phases.is_none());
        assert!(c_v1_to_v2.dependencies.is_none());
        assert!(c_v1_to_v2.job_dispatch.is_none());
    });

    test!(load_config_failure_invalid_config {
        let source_dir = fixture_file("test_bug_repo");
        assert!(DeliveryConfig::load_config(&source_dir).is_err());
    });


    test!(validate_config_file {
        let source_dir = fixture_file("test_repo");
        assert!(DeliveryConfig::validate_config_file(&source_dir).unwrap());
    });

    test!(load_complex_config {
        let source_dir = fixture_file("test_complex_repo");
        let c_v2 = DeliveryConfig::load_config(&source_dir).unwrap();
        let mut build_nodes = HashMap::new();
        build_nodes.insert(
            "default".to_string(), vec!["name:delivery-builder*".to_string()]
        );
        let mut build_cookbook = HashMap::new();
        build_cookbook.insert("name".to_string(),
                              "cerebro".to_string());
        build_cookbook.insert("enterprise".to_string(),
                              "marvel".to_string());
        build_cookbook.insert("organization".to_string(),
                              "x-men".to_string());

        assert_eq!(c_v2.version, "2".to_string());
        assert_eq!(
            c_v2.skip_phases,
            Some(
                vec!["unit".to_string(),
                "deploy".to_string(),
                "quality".to_string()]
            )
        );
        assert_eq!(
            c_v2.dependencies,
            Some(vec!["projectA".to_string(), "projectZ".to_string()])
        );
        assert_eq!(c_v2.build_cookbook, build_cookbook);
        assert!(c_v2.build_nodes.is_none());

        assert!(c_v2.job_dispatch.is_some());
        let complex_job_disp = c_v2.job_dispatch.unwrap();
        assert_eq!(complex_job_disp.version, "v2".to_string());

        assert!(complex_job_disp.filters.is_some());
        let complex_filters = complex_job_disp.filters.unwrap();

        // Extract the platform_family content of the filter1 for the syntax phase
        assert_eq!(complex_filters.get("syntax")
                    .unwrap()[0]
                    .get("platform_family")
                    .unwrap()[0],
                    String::from("debian"));
        // Extract the platform_version content of the filter2 for the publish phase
        assert_eq!(complex_filters.get("publish")
                    .unwrap()[1]
                    .get("platform_version")
                    .unwrap()[0],
                    String::from("14.04"));
        // Extract the platform_family content of the filter3 for the unit phase
        assert_eq!(complex_filters.get("unit")
                    .unwrap()[2]
                    .get("platform_family")
                    .unwrap()[0],
                    String::from("rhel"));

        // Filter for deploy phase doesn't exist
        assert!(complex_filters.get("deploy").is_none());
    });
}
