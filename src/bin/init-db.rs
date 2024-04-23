use rusty_router::db;

fn main() {
    db::create_tables().unwrap();

    let neighbors = db::get_neighbors(0).unwrap();
    println!("neighbors: {neighbors:?}");
}
