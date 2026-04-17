pub mod gitlab_client {
    use crate::CiServer;
    use gitlab::api::projects::pipelines::Pipelines;
    use gitlab::api::Query;
    use gitlab::{api, Gitlab};
    use serde::{Deserialize, Serialize};
    use std::collections::HashSet;
    use std::thread;
    use std::time::Duration;
    use tracing::{debug, info, warn};
    use url::Url;

    const MAX_RETRIES: u32 = 3;
    const INITIAL_BACKOFF_MS: u64 = 1000;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct PipelineData {
        pub status: String,
        pub web_url: String,
        pub r#ref: String,
        pub id: u32,
        pub created_at: Option<String>,
        pub updated_at: Option<String>,
        pub finished_at: Option<String>,
        pub sha: Option<String>,
        pub source: Option<String>,
    }

    fn extract_host(url_string: &str) -> Result<String, String> {
        let url = Url::parse(url_string)
            .map_err(|e| format!("Invalid URL '{}': {}", url_string, e))?;

        url.host_str()
            .map(|s| s.to_string())
            .ok_or_else(|| format!("URL '{}' has no host", url_string))
    }

    fn create_gitlab_client(ci_server: &CiServer) -> Result<Gitlab, String> {
        let host = extract_host(&ci_server.url_string)?;
        debug!("Creating GitLab client for host: {}", host);

        Gitlab::new(&host, &ci_server.api_key)
            .map_err(|e| format!("Failed to create GitLab client: {}", e))
    }

    fn with_retry<T, F>(operation_name: &str, operation: F) -> Result<T, String>
    where
        F: Fn() -> Result<T, String>,
    {
        let mut last_error = String::new();

        for attempt in 0..MAX_RETRIES {
            match operation() {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = e.clone();

                    // Check for rate limiting in error message
                    if e.contains("429") || e.to_lowercase().contains("rate limit") {
                        warn!(
                            "{} rate limited (attempt {}), waiting before retry",
                            operation_name,
                            attempt + 1
                        );
                        // Longer wait for rate limiting
                        thread::sleep(Duration::from_secs(30));
                    } else if attempt < MAX_RETRIES - 1 {
                        let backoff = INITIAL_BACKOFF_MS * 2u64.pow(attempt);
                        warn!(
                            "{} failed (attempt {}): {}. Retrying in {}ms",
                            operation_name,
                            attempt + 1,
                            e,
                            backoff
                        );
                        thread::sleep(Duration::from_millis(backoff));
                    } else {
                        warn!(
                            "{} failed (attempt {}): {}. No more retries.",
                            operation_name,
                            attempt + 1,
                            e
                        );
                    }
                }
            }
        }

        Err(format!(
            "{} failed after {} retries: {}",
            operation_name, MAX_RETRIES, last_error
        ))
    }

    pub fn get_gitlab_pipelines(
        reference: &str,
        project_name: &str,
        ci_server: &CiServer,
    ) -> Result<Vec<PipelineData>, String> {
        debug!(
            "Fetching pipelines for project '{}', ref '{}'",
            project_name, reference
        );

        let client = create_gitlab_client(ci_server)?;
        let project_name = project_name.to_string();
        let reference = reference.to_string();

        with_retry("get_gitlab_pipelines", move || {
            let pipelines = Pipelines::builder()
                .project(&project_name)
                .ref_(&reference)
                .build()
                .map_err(|e| format!("Failed to build pipeline query: {}", e))?;

            let result: Vec<PipelineData> = pipelines
                .query(&client)
                .map_err(|e| format!("Failed to query pipelines: {}", e))?;

            info!(
                "Retrieved {} pipelines for project '{}', ref '{}'",
                result.len(),
                project_name,
                reference
            );
            Ok(result)
        })
    }

    pub fn get_references(
        project_name: &str,
        ci_server: &CiServer,
    ) -> Result<Vec<String>, String> {
        use gitlab::api::projects::repository::branches::Branches;
        use gitlab::api::projects::repository::tags::Tags;

        #[derive(Deserialize)]
        struct NamedRef {
            name: String,
        }

        debug!("Fetching references for project '{}'", project_name);

        let client = create_gitlab_client(ci_server)?;
        let project_name = project_name.to_string();

        with_retry("get_references", move || {
            let branches_req = Branches::builder()
                .project(&project_name)
                .build()
                .map_err(|e| format!("Failed to build branches query: {}", e))?;
            let branches: Vec<NamedRef> = api::paged(branches_req, api::Pagination::Limit(500))
                .query(&client)
                .map_err(|e| format!("Failed to query branches: {}", e))?;

            let tags_req = Tags::builder()
                .project(&project_name)
                .build()
                .map_err(|e| format!("Failed to build tags query: {}", e))?;
            let tags: Vec<NamedRef> = api::paged(tags_req, api::Pagination::Limit(500))
                .query(&client)
                .map_err(|e| format!("Failed to query tags: {}", e))?;

            let refs_set: HashSet<String> = branches
                .into_iter()
                .chain(tags)
                .map(|r| r.name)
                .collect();
            let mut refs_list: Vec<_> = refs_set.into_iter().collect();
            refs_list.sort();

            info!(
                "Retrieved {} unique references for project '{}'",
                refs_list.len(),
                project_name
            );
            Ok(refs_list)
        })
    }
}
