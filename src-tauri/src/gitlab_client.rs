pub mod gitlab_client {
    use std::collections::HashSet;
    use ::gitlab::api::projects::pipelines::Pipelines;
    use ::gitlab::api::Query;
    use gitlab::{api, Gitlab};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub(crate) struct PipelineData {
        status: String,
        web_url: String,
        r#ref: String,
        id: u32,
        created_at: Option<String>,
        updated_at: Option<String>,
        finished_at: Option<String>,
    }

    pub(crate) fn get_gitlab_pipelines(reference: &str, project_name: &str, client: &Gitlab) -> Vec<PipelineData> {
        let pipelines = Pipelines::builder()
            .project(project_name)
            .ref_(reference)
            .build()
            .unwrap();
        // let project: Vec<PipelineData> = api::paged(pipelines, api::Pagination::Limit(500))
        let project: Vec<PipelineData> = pipelines
            .query(client)
            .unwrap();
        // let refs: HashSet<String> = project.into_iter().map(|x| x.r#ref).collect();
        return project;
    }

    pub(crate) fn get_references(project_name: &str, client: &Gitlab) -> Vec<String> {
        let req = Pipelines::builder()
            .project(project_name)
            .build()
            .unwrap();
        let refs: Vec<PipelineData> = api::paged(req, api::Pagination::Limit(500))
            .query(client).unwrap();
        let refs_set: HashSet<String> = refs.into_iter().map(|x| x.r#ref).collect();
        let mut refs_list = refs_set.into_iter().map(String::from).collect::<Vec<_>>();
        refs_list.sort();
        return refs_list;
    }
}
