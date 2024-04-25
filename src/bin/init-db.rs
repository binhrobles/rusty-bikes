use rusty_router::db;

fn main() {
    let conn = db::get_conn().unwrap();
    db::init_tables(&conn).unwrap();
}
