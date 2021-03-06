#[cfg(feature = "itest")]
#[cfg(test)]
mod tests {

    use crate::utils::{
        build, build_api_client, create_namespace, create_project, delete_build_cache,
        get_build_metadata, list_manifest_metadata, list_namespace, list_project, push_manifest,
        scan_build_cache_metadata, wait,
    };
    use pipebuilder_common::{read_file, BuildStatus};

    const TEST_CLUSTER_READY_MILLIS: u64 = 30000;
    const TEST_NAMESPACE: &str = "dev";
    const TEST_PROJECT: &str = "timer";
    const TEST_BUILD_COUNT: u64 = 3;
    const TEST_BUILD_WAIT_MILLIS: u64 = 30000;
    const TEST_BUILD_WAIT_RETRY: u64 = 5;
    const TEST_BUILDER_ID: &str = "builder0";
    const TEST_TARGET_PLATFORM: &str = "x86_64-unknown-linux-gnu";

    #[tokio::test]
    async fn test_build() {
        wait(TEST_CLUSTER_READY_MILLIS).await;
        let client = build_api_client("resources/cli.yml").await.unwrap();
        // create namespace
        let expected_namespace = create_namespace(&client, String::from(TEST_NAMESPACE))
            .await
            .unwrap();
        let namespaces = list_namespace(&client).await.unwrap();
        assert_eq!(1, namespaces.len());
        let actual_namespace = namespaces.get(0).unwrap();
        assert_eq!(expected_namespace.id, actual_namespace.id);

        // create project
        let expected_project = create_project(
            &client,
            String::from(TEST_NAMESPACE),
            String::from(TEST_PROJECT),
        )
        .await
        .unwrap();
        let projects = list_project(&client, String::from(TEST_NAMESPACE))
            .await
            .unwrap();
        assert_eq!(1, projects.len());
        let actual_project = projects.get(0).unwrap();
        assert_eq!(expected_project.id, actual_project.id);

        // push manifest
        let buffer = read_file("tests/timer/pipe.yml").await.unwrap();
        let _ = push_manifest(
            &client,
            String::from(TEST_NAMESPACE),
            String::from(TEST_PROJECT),
            buffer,
        )
        .await
        .unwrap();
        let manifest_metadatas = list_manifest_metadata(
            &client,
            String::from(TEST_NAMESPACE),
            Some(String::from(TEST_PROJECT)),
        )
        .await
        .unwrap();
        assert_eq!(1, manifest_metadatas.len());
        let actual_manifest_metadata = manifest_metadatas.get(0).unwrap();
        assert_eq!(String::from(TEST_PROJECT), actual_manifest_metadata.id);
        assert_eq!(0, actual_manifest_metadata.version);
        // no build cache
        let build_caches = scan_build_cache_metadata(&client, String::from(TEST_BUILDER_ID))
            .await
            .unwrap();
        assert!(build_caches.is_empty());
        for i in 0..TEST_BUILD_COUNT {
            // build
            let build_response = build(
                &client,
                String::from(TEST_NAMESPACE),
                String::from(TEST_PROJECT),
                0,
                None,
            )
            .await
            .unwrap();
            let build_version = build_response.build_version;
            assert_eq!(i, build_version);
            wait(TEST_BUILD_WAIT_MILLIS).await;
            for j in 0..TEST_BUILD_WAIT_RETRY {
                let build_metadata = get_build_metadata(
                    &client,
                    String::from(TEST_NAMESPACE),
                    String::from(TEST_PROJECT),
                    build_version,
                )
                .await
                .unwrap();
                let status = build_metadata.status;
                match status {
                    BuildStatus::Cancel | BuildStatus::Fail | BuildStatus::Succeed => break,
                    _ => {
                        println!(
                            "build in progress, retry({}) in {} millis",
                            j, TEST_BUILD_WAIT_MILLIS
                        );
                        wait(TEST_BUILD_WAIT_MILLIS).await;
                    }
                }
            }
            let build_metadata = get_build_metadata(
                &client,
                String::from(TEST_NAMESPACE),
                String::from(TEST_PROJECT),
                build_version,
            )
            .await
            .unwrap();
            let status = build_metadata.status;
            assert!(matches!(status, BuildStatus::Succeed));
            // build cache hit
            let build_caches = scan_build_cache_metadata(&client, String::from(TEST_BUILDER_ID))
                .await
                .unwrap();
            assert_eq!(1, build_caches.len());
        }
        // cleanup build cache
        delete_build_cache(
            &client,
            String::from(TEST_BUILDER_ID),
            String::from(TEST_NAMESPACE),
            String::from(TEST_PROJECT),
            String::from(TEST_TARGET_PLATFORM),
        )
        .await
        .unwrap();
        let build_caches = scan_build_cache_metadata(&client, String::from(TEST_BUILDER_ID))
            .await
            .unwrap();
        assert!(build_caches.is_empty());
    }
}
