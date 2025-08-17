pub mod database {
    use crate::CiServer;
    use rusqlite::{Connection, Result};

    static PATH: &'static str = "cignaler.db";

    pub fn init_db() -> Result<()> {
        let conn = Connection::open(PATH)?;

        conn.execute(
            "create table if not exists ci_servers (
             name text not null primary key,
             server_type text not null,
             url_string text not null,
             api_key text not null
         )",
            {},
        )?;
        conn.execute(
            "create table if not exists listeners (
             id integer primary key,
             name text not null,
             color_id integer not null references cat_colors(id)
         )",
            {},
        )?;

        conn.close()
            .inspect_err(|err| println!("DB initialization failed: {err:#?}"));

        println!("create_db success");
        Ok(())
    }
    pub fn save_ci_server_data(
        name: String,
        server_type: String,
        url_string: String,
        api_key: String,
    ) -> Result<()> {
        let conn = Connection::open(PATH)?;

        conn.execute(
            "insert into ci_servers (name, server_type, url_string, api_key) VALUES (?1, ?2, ?3, ?4)",
            (&name, &server_type, &url_string, &api_key),
        ).inspect_err(|e| println!("failed to store: {}", e));

        conn.close()
            .inspect_err(|err| println!("DB initialization failed: {err:#?}"));

        Ok(())
    }

    pub fn read_ci_servers_data() -> Result<Vec<CiServer>> {
        let conn = Connection::open(PATH)?;
        let mut servers = Vec::new();
        conn.prepare("select name, server_type, url_string, api_key from ci_servers")
            .unwrap()
            .query_map([], |row| CiServer::from_row(row))?
            .for_each(|x| servers.push(x.unwrap()));

        Ok(servers)
    }
}
