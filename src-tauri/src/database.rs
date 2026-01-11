pub mod database {
    use crate::error::{CignalerError, Result};
    use crate::CiServer;
    use r2d2::{Pool, PooledConnection};
    use r2d2_sqlite::SqliteConnectionManager;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::OnceLock;
    use tracing::{debug, info};

    static DB_POOL: OnceLock<Pool<SqliteConnectionManager>> = OnceLock::new();

    fn get_db_path() -> PathBuf {
        let data_dir = dirs::data_local_dir()
            .or_else(|| dirs::home_dir())
            .unwrap_or_else(|| PathBuf::from("."));

        let app_dir = data_dir.join("cignaler");

        if !app_dir.exists() {
            let _ = fs::create_dir_all(&app_dir);
        }

        app_dir.join("cignaler.db")
    }

    fn get_connection() -> Result<PooledConnection<SqliteConnectionManager>> {
        DB_POOL
            .get()
            .ok_or_else(|| CignalerError::Config("Database pool not initialized".into()))?
            .get()
            .map_err(CignalerError::from)
    }

    pub fn init_db() -> Result<()> {
        let db_path = get_db_path();
        info!("Initializing database at: {:?}", db_path);

        let manager = SqliteConnectionManager::file(&db_path);
        let pool = Pool::builder()
            .max_size(10)
            .build(manager)
            .map_err(|e| CignalerError::Config(format!("Failed to create connection pool: {}", e)))?;

        // Initialize schema using a connection from the pool
        {
            let conn = pool.get()?;

            // Enable foreign key support
            conn.execute("PRAGMA foreign_keys = ON", [])?;

            conn.execute(
                "CREATE TABLE IF NOT EXISTS ci_servers (
                    name TEXT NOT NULL PRIMARY KEY,
                    server_type TEXT NOT NULL,
                    url_string TEXT NOT NULL,
                    api_key TEXT NOT NULL
                )",
                [],
            )?;

            conn.execute(
                "CREATE TABLE IF NOT EXISTS projects (
                    id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL,
                    ci_server_name TEXT NOT NULL,
                    project_path TEXT NOT NULL,
                    default_branch TEXT,
                    enabled BOOLEAN DEFAULT 1,
                    FOREIGN KEY (ci_server_name) REFERENCES ci_servers(name) ON DELETE CASCADE
                )",
                [],
            )?;

            // Run migrations
            migrate_db(&conn)?;
        }

        DB_POOL
            .set(pool)
            .map_err(|_| CignalerError::Config("Database pool already initialized".into()))?;

        info!("Database initialized successfully");
        Ok(())
    }

    fn migrate_db(conn: &rusqlite::Connection) -> Result<()> {
        let version: i32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
        debug!("Current database version: {}", version);

        if version < 1 {
            info!("Running migration to version 1: Adding ON DELETE CASCADE");

            // Check if we need to migrate (old table without CASCADE)
            let table_info: String = conn.query_row(
                "SELECT sql FROM sqlite_master WHERE type='table' AND name='projects'",
                [],
                |row| row.get(0),
            ).unwrap_or_default();

            if !table_info.contains("ON DELETE CASCADE") && !table_info.is_empty() {
                conn.execute_batch(
                    "PRAGMA foreign_keys=OFF;

                    CREATE TABLE IF NOT EXISTS projects_new (
                        id INTEGER PRIMARY KEY,
                        name TEXT NOT NULL,
                        ci_server_name TEXT NOT NULL,
                        project_path TEXT NOT NULL,
                        default_branch TEXT,
                        enabled BOOLEAN DEFAULT 1,
                        FOREIGN KEY (ci_server_name) REFERENCES ci_servers(name) ON DELETE CASCADE
                    );

                    INSERT OR IGNORE INTO projects_new SELECT * FROM projects;
                    DROP TABLE IF EXISTS projects;
                    ALTER TABLE projects_new RENAME TO projects;

                    PRAGMA foreign_keys=ON;"
                )?;
                info!("Migration to version 1 completed");
            }

            conn.execute("PRAGMA user_version = 1", [])?;
        }

        Ok(())
    }

    pub fn save_ci_server_data(
        name: String,
        server_type: String,
        url_string: String,
        api_key: String,
    ) -> Result<()> {
        let conn = get_connection()?;
        debug!("Saving CI server: name={}, type={}", name, server_type);

        conn.execute(
            "INSERT INTO ci_servers (name, server_type, url_string, api_key) VALUES (?1, ?2, ?3, ?4)",
            (&name, &server_type, &url_string, &api_key),
        )?;

        info!("CI server '{}' saved successfully", name);
        Ok(())
    }

    pub fn update_ci_server_data(
        name: String,
        server_type: String,
        url_string: String,
        api_key: String,
    ) -> Result<()> {
        let conn = get_connection()?;
        debug!("Updating CI server: name={}", name);

        conn.execute(
            "UPDATE ci_servers SET server_type = ?1, url_string = ?2, api_key = ?3 WHERE name = ?4",
            (&server_type, &url_string, &api_key, &name),
        )?;

        info!("CI server '{}' updated successfully", name);
        Ok(())
    }

    pub fn read_ci_servers_data() -> Result<Vec<CiServer>> {
        let conn = get_connection()?;
        let mut stmt = conn.prepare("SELECT name, server_type, url_string, api_key FROM ci_servers")?;
        let servers = stmt
            .query_map([], |row| CiServer::from_row(row))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        debug!("Read {} CI servers from database", servers.len());
        Ok(servers)
    }

    pub fn delete_ci_server_data(name: String) -> Result<()> {
        let conn = get_connection()?;
        debug!("Deleting CI server: name={}", name);

        // Foreign key cascade will automatically delete associated projects
        conn.execute("DELETE FROM ci_servers WHERE name = ?1", (&name,))?;

        info!("CI server '{}' deleted successfully (with cascaded projects)", name);
        Ok(())
    }

    pub fn save_project_data(
        name: String,
        ci_server_name: String,
        project_path: String,
        default_branch: Option<String>,
    ) -> Result<()> {
        let conn = get_connection()?;
        debug!("Saving project: name={}, server={}", name, ci_server_name);

        conn.execute(
            "INSERT INTO projects (name, ci_server_name, project_path, default_branch) VALUES (?1, ?2, ?3, ?4)",
            (&name, &ci_server_name, &project_path, &default_branch),
        )?;

        info!("Project '{}' saved successfully", name);
        Ok(())
    }

    pub fn read_projects_data() -> Result<Vec<crate::CiProject>> {
        let conn = get_connection()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, ci_server_name, project_path, default_branch, enabled FROM projects",
        )?;
        let projects = stmt
            .query_map([], |row| crate::CiProject::from_row(row))?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        debug!("Read {} projects from database", projects.len());
        Ok(projects)
    }

    pub fn update_project_data(
        id: i64,
        name: String,
        ci_server_name: String,
        project_path: String,
        default_branch: Option<String>,
    ) -> Result<()> {
        let conn = get_connection()?;
        debug!("Updating project: id={}, name={}", id, name);

        conn.execute(
            "UPDATE projects SET name = ?1, ci_server_name = ?2, project_path = ?3, default_branch = ?4 WHERE id = ?5",
            (&name, &ci_server_name, &project_path, &default_branch, &id),
        )?;

        info!("Project '{}' (id={}) updated successfully", name, id);
        Ok(())
    }

    pub fn update_project_enabled(id: i64, enabled: bool) -> Result<()> {
        let conn = get_connection()?;
        debug!("Updating project enabled state: id={}, enabled={}", id, enabled);

        conn.execute(
            "UPDATE projects SET enabled = ?1 WHERE id = ?2",
            (enabled, id),
        )?;

        info!("Project id={} enabled state set to {}", id, enabled);
        Ok(())
    }

    pub fn delete_project_data(id: i64) -> Result<()> {
        let conn = get_connection()?;
        debug!("Deleting project: id={}", id);

        conn.execute("DELETE FROM projects WHERE id = ?1", (&id,))?;

        info!("Project id={} deleted successfully", id);
        Ok(())
    }
}
