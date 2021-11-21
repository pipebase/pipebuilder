#[cfg(feature = "itest")]
#[cfg(test)]
mod tests {

    use crate::utils::{
        build_api_client, list_api_state, list_builder_state, list_repository_state,
        list_scheduler_state, shutdown_ci,
    };
    use pipebuilder_common::NodeRole;

    #[tokio::test]
    async fn test_node() {
        let client = build_api_client("resources/cli.yml").await.unwrap();
        // validate api
        let node_states = list_api_state(&client).await.unwrap();
        assert_eq!(1, node_states.len());
        let node_state = node_states.get(0).unwrap();
        assert_eq!("api0", node_state.id);
        assert!(matches!(node_state.role, NodeRole::Api));

        // validate builder
        let node_states = list_builder_state(&client).await.unwrap();
        assert_eq!(1, node_states.len());
        let node_state = node_states.get(0).unwrap();
        assert_eq!("builder0", node_state.id);
        assert!(matches!(node_state.role, NodeRole::Builder));

        // validate repository
        let node_states = list_repository_state(&client).await.unwrap();
        assert_eq!(1, node_states.len());
        let node_state = node_states.get(0).unwrap();
        assert_eq!("repository0", node_state.id);
        assert!(matches!(node_state.role, NodeRole::Repository));

        // validate scheduler
        let node_states = list_scheduler_state(&client).await.unwrap();
        assert_eq!(1, node_states.len());
        let node_state = node_states.get(0).unwrap();
        assert_eq!("scheduler0", node_state.id);
        assert!(matches!(node_state.role, NodeRole::Scheduler));

        shutdown_ci(&client).await.unwrap();
    }
}
