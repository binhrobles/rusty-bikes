use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sqlx::{
    Connection,
    SqliteConnection,
    types::Json,
}; 

type NodeId = String;
type WayId = String;

// this type declaration was preferable to defining a newtype struct
// because sqlx has some chunky ergonomics around nested types
// while JSON has first class handling
type Neighbors = HashMap<NodeId, WayId>;

#[derive(Debug, Serialize, Deserialize)]
struct Node {
    lat: f32,
    long: f32,
    id: NodeId,
    neighbors: Json<Neighbors>,
}

/// initializes a sqlite database at DATABASE_URL with the requisite tables
pub async fn create_tables() -> Result<(), anyhow::Error> {
    let mut conn = SqliteConnection::connect(&std::env::var("DATABASE_URL").unwrap()).await?;

    sqlx::query("
        DROP TABLE IF EXISTS Node;
        CREATE TABLE Node (
            id TEXT PRIMARY KEY,
            lat REAL NOT NULL,
            long REAL NOT NULL,
            neighbors TEXT
        )
    ").execute(&mut conn).await?;
    println!("Table created");

    let seed = Node {
        id: "0".to_owned(),
        lat: 40.5,
        long: 70.5,
        neighbors: Json(HashMap::from([
                        ("1".to_owned(), "1".to_owned()),
                        ("2".to_owned(), "2".to_owned()),
                    ])
                ),
    };

    sqlx::query("INSERT INTO Node (id, lat, long, neighbors) VALUES (?1, ?2, ?3, ?4)")
        .bind(seed.id)
        .bind(seed.lat)
        .bind(seed.long)
        .bind(seed.neighbors)
        .execute(&mut conn).await?;

    Ok(())
}

pub async fn get_neighbors(id: NodeId) -> Result<Neighbors, anyhow::Error> {
    let mut conn = SqliteConnection::connect(&std::env::var("DATABASE_URL").unwrap()).await?;

    let n: Json<Neighbors> = sqlx::query_scalar("SELECT neighbors FROM Node WHERE id = ?1")
        .bind(id)
        .fetch_one(&mut conn)
        .await?;

    // TODO: is there a more idiomatic way to get the HashMap out of the sqlx Json wrapper?
    Ok(n.as_ref().clone())
}
