pub mod models {
    use rusqlite::{Result, Row};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct CiServer {
        pub name: String,
        pub server_type: String,
        pub url_string: String,
        pub api_key: String,
    }

    impl CiServer {
        pub fn from_row(row: &Row) -> Result<CiServer> {
            Ok(CiServer {
                name: row.get(0)?,
                server_type: row.get(1)?,
                url_string: row.get(2)?,
                api_key: row.get(3)?,
            })
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct CiProject {
        pub id: i64,
        pub name: String,
        pub ci_server_name: String,
        pub project_path: String,
        pub default_branch: Option<String>,
        pub enabled: bool,
    }

    impl CiProject {
        pub fn from_row(row: &Row) -> Result<CiProject> {
            Ok(CiProject {
                id: row.get(0)?,
                name: row.get(1)?,
                ci_server_name: row.get(2)?,
                project_path: row.get(3)?,
                default_branch: row.get(4)?,
                enabled: row.get(5)?,
            })
        }
    }
}
