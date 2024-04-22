use rusty_router::db;

fn main() {
    db::create_tables().unwrap();
}
