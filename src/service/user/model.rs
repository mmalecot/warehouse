use crate::database::{schema::warehouse_user, Connection};
use chrono::NaiveDateTime;
use diesel::{
    dsl, result::Error, BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl,
    RunQueryDsl,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Insertable, Queryable, Serialize)]
#[table_name = "warehouse_user"]
pub struct User {
    pub id: String,
    pub creation_date: NaiveDateTime,
    pub name: String,
    pub email: String,
    pub password: String,
    pub admin: bool,
}

impl User {
    pub fn count(connection: &Connection) -> Result<i64, Error> {
        Ok(warehouse_user::table
            .select(dsl::count_star())
            .first(connection)?)
    }

    pub fn create(&self, connection: &Connection) -> Result<(), Error> {
        dsl::insert_into(warehouse_user::table)
            .values(self)
            .execute(connection)?;
        Ok(())
    }

    pub fn exists(connection: &Connection, name: &str, email: &str) -> Result<bool, Error> {
        Ok(warehouse_user::table
            .filter(
                warehouse_user::name
                    .eq(name)
                    .or(warehouse_user::email.eq(email)),
            )
            .first::<User>(connection)
            .optional()?
            .is_some())
    }

    pub fn find_by_id(connection: &Connection, id: &str) -> Result<Option<User>, Error> {
        Ok(warehouse_user::table
            .find(id)
            .first(connection)
            .optional()?)
    }

    pub fn find_by_name_or_email(
        connection: &Connection,
        field: &str,
    ) -> Result<Option<User>, Error> {
        Ok(warehouse_user::table
            .filter(
                warehouse_user::name
                    .eq(field)
                    .or(warehouse_user::email.eq(field)),
            )
            .first(connection)
            .optional()?)
    }
}
