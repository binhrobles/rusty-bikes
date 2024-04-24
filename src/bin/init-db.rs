use rusty_router::db;

fn main() {
    let conn = db::get_conn().unwrap();
    db::create_tables(&conn).unwrap();

    let neighbors = db::get_neighbors(&conn, 0).unwrap();
    println!("neighbors: {neighbors:?}");
}
