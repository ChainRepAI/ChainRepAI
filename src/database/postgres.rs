use diesel::PgConnection;

pub struct Database {
    conn: PgConnection,
}
