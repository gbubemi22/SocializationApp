use mongodb::{Client, options::ClientOptions};
use mongodb::bson::doc;
use std::error::Error;

pub struct Database {
    pub client: Client,
}

impl Database {
    pub async fn init() -> Result<Self, Box<dyn Error>> {
        let mongodb_uri = std::env::var("MONGODB_URI")
            .unwrap_or_else(|_| "mongodb://localhost:27017".to_string());

        let mut client_options = ClientOptions::parse(&mongodb_uri).await?;
        client_options.app_name = Some("rust_project".to_string());

        let client = Client::with_options(client_options)?;

        // Ping the server to see if you can connect to the cluster
        client
            .database("admin")
            .run_command(doc! {"ping": 1}, )
            .await?;

        println!("Connected successfully to MongoDB");

        Ok(Self { client })
    }

    // You can add more database-related methods here
}

// This function is a convenience wrapper around Database::init()
pub async fn connect_to_mongo() -> Result<Client, Box<dyn Error>> {
     let database = Database::init().await.map_err(|e| {
          eprintln!("Failed to initialize database: {:?}", e);
          e
      })?;
      Ok(database.client)
}