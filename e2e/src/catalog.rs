#[cfg(feature = "itest")]
#[cfg(test)]
mod tests {

    use crate::utils::{
        build_api_client, create_namespace, create_project, list_catalog_schema_metadata,
        list_catalogs_metadata, list_namespace, list_project, pull_catalog_schema, pull_catalogs,
        push_catalog_schema, push_catalogs,
    };
    use pipebuilder_common::read_file;

    const TEST_NAMESPACE: &str = "dev";
    const TEST_PROJECT: &str = "timer";
    const TEST_CATALOG_SCHEMA: &str = "timer_schema";

    #[tokio::test]
    async fn test_catalog() {
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

        // push catalog schema
        let buffer = read_file("tests/timer/catalog_schemas/timer_schema.json")
            .await
            .unwrap();
        let resp = push_catalog_schema(
            &client,
            String::from(TEST_NAMESPACE),
            String::from(TEST_CATALOG_SCHEMA),
            buffer.clone(),
        )
        .await
        .unwrap();
        let expected_version = resp.version;
        let metadatas = list_catalog_schema_metadata(
            &client,
            String::from(TEST_NAMESPACE),
            Some(String::from(TEST_CATALOG_SCHEMA)),
        )
        .await
        .unwrap();
        assert_eq!(1, metadatas.len());
        let metadata = metadatas.get(0).unwrap();
        assert_eq!(expected_version, metadata.version);
        // compare binaries
        let resp = pull_catalog_schema(
            &client,
            String::from(TEST_NAMESPACE),
            String::from(TEST_CATALOG_SCHEMA),
            expected_version,
        )
        .await
        .unwrap();
        let actual_buffer = resp.buffer;
        assert_eq!(buffer, actual_buffer);
        // push and validate catalogs
        let buffer = read_file("tests/timer/catalogs/catalogs.yml")
            .await
            .unwrap();
        let resp = push_catalogs(
            &client,
            String::from(TEST_NAMESPACE),
            String::from(TEST_PROJECT),
            buffer.clone(),
        )
        .await
        .unwrap();
        let expected_version = resp.version;
        let metadatas = list_catalogs_metadata(
            &client,
            String::from(TEST_NAMESPACE),
            Some(String::from(TEST_PROJECT)),
        )
        .await
        .unwrap();
        assert_eq!(1, metadatas.len());
        let metadata = metadatas.get(0).unwrap();
        assert_eq!(expected_version, metadata.version);
        // compare binaries
        let resp = pull_catalogs(
            &client,
            String::from(TEST_NAMESPACE),
            String::from(TEST_PROJECT),
            expected_version,
        )
        .await
        .unwrap();
        let actual_buffer = resp.buffer;
        assert_eq!(buffer, actual_buffer);
    }
}
