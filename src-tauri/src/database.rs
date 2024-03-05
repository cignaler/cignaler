pub mod database {
    use rusqlite::{Connection, Result};

    pub fn init_db() -> Result<()> {
        let conn = Connection::open("cignaler.db")?;

        conn.execute(
            "create table if not exists ci_servers (
             id integer primary key,
             name text not null unique,
             type text not null,
             url text not null,
             secret text not null
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

        println!("create_db success");
        Ok(())
    }
}
