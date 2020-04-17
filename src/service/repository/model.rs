use crate::database::{schema::warehouse_repository, Connection};
use diesel::{result::Error, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Insertable, Queryable, Serialize)]
#[table_name = "warehouse_repository"]
pub struct Repository {
    pub id: String,
    pub name: String,
    pub extension: String,
}

impl Repository {
    pub fn find_by_name(connection: &Connection, name: &str) -> Result<Option<Repository>, Error> {
        Ok(warehouse_repository::table
            .filter(warehouse_repository::name.eq(name))
            .first(connection)
            .optional()?)
    }

    pub fn list(connection: &Connection) -> Result<Vec<Repository>, Error> {
        Ok(warehouse_repository::table
            .select(warehouse_repository::all_columns)
            .load(connection)?)
    }
}
