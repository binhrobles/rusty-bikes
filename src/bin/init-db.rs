use rusty_router::db;
use dotenvy::dotenv;

#[tokio::main]
async fn main() {
    dotenv().expect(".env file not found");

    match db::create_tables() {
        Ok(_) => {}
        Err(e) => {
            println!("init error: {:?}", e);
        }
    }
}
