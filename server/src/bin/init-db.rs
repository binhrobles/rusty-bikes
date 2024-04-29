use rusty_router::osm::db;

fn main() {
    let conn = db::get_conn().unwrap();
    db::init_tables(&conn).unwrap();
}
