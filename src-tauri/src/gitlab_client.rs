pub mod gitlab_client {
    use std::collections::HashSet;
    use ::gitlab::api::projects::pipelines::Pipelines;
    use ::gitlab::api::Query;
    use gitlab::{api, Gitlab};
    use serde::{Deserialize, Serialize};
    use crate::CiServer;

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

    pub(crate) fn get_gitlab_pipelines(
        reference: &str, 
        project_name: &str, 
        ci_server: &CiServer
    ) -> Result<Vec<PipelineData>, String> {
        // Extract host from URL (remove https:// prefix if present)
        let host = ci_server.url_string.strip_prefix("https://").unwrap_or(&ci_server.url_string);
        let host = host.strip_prefix("http://").unwrap_or(host);
        
        let client = Gitlab::new(host, &ci_server.api_key)
            .map_err(|e| format!("Failed to create GitLab client: {}", e))?;

        let pipelines = Pipelines::builder()
            .project(project_name)
            .ref_(reference)
            .build()
            .map_err(|e| format!("Failed to build pipeline query: {}", e))?;

        let project: Vec<PipelineData> = pipelines
            .query(&client)
            .map_err(|e| format!("Failed to query pipelines: {}", e))?;

        Ok(project)
    }

    pub(crate) fn get_references(
        project_name: &str, 
        ci_server: &CiServer
    ) -> Result<Vec<String>, String> {
        // Extract host from URL (remove https:// prefix if present)
        let host = ci_server.url_string.strip_prefix("https://").unwrap_or(&ci_server.url_string);
        let host = host.strip_prefix("http://").unwrap_or(host);
        
        let client = Gitlab::new(host, &ci_server.api_key)
            .map_err(|e| format!("Failed to create GitLab client: {}", e))?;

        let req = Pipelines::builder()
            .project(project_name)
            .build()
            .map_err(|e| format!("Failed to build pipeline query: {}", e))?;

        let refs: Vec<PipelineData> = api::paged(req, api::Pagination::Limit(500))
            .query(&client)
            .map_err(|e| format!("Failed to query pipeline references: {}", e))?;

        let refs_set: HashSet<String> = refs.into_iter().map(|x| x.r#ref).collect();
        let mut refs_list = refs_set.into_iter().collect::<Vec<_>>();
        refs_list.sort();
        Ok(refs_list)
    }
}
