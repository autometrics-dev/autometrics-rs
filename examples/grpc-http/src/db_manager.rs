use std::fmt::Error;

// Clone is required for the `tokio::signal::unix::SignalKind::terminate()` handler
// Sometimes, you can't derive clone, then you have to wrap the DBManager in an Arc or Arc<Mutex>
#[derive(Debug, Default, Clone, Copy)]
pub struct DBManager {
    // Put your DB client here. For example:
    // db: rusqlite,
}

impl DBManager {
    pub fn new() -> DBManager {
        DBManager {
            // Put your database client here. For example:
            // db: rusqlite::Connection::open(":memory:").unwrap();
        }
    }

   pub  async fn connect_to_db(&self) -> Result<(), Error>{
        println!("Connecting to database");
        Ok(())
    }

    pub async fn close_db(&self) -> Result<(), Error>{
        println!("DB connection closed");
        Ok(())
    }

    pub async fn query_table(&self) -> Result<(), Error>{
        println!("Query table");
        Ok(())
    }

    pub async fn write_into_table(&self) -> Result<(), Error>{
        println!("Write into table");
        Ok(())
    }
}
