use crate::{
    database::{
        schema::{
            warehouse_package, warehouse_package_dependency, warehouse_package_file,
            warehouse_package_version, warehouse_repository, warehouse_user,
        },
        Connection,
    },
    service::{repository::model::Repository, user::model::User},
};
use chrono::NaiveDateTime;
use diesel::{
    dsl, result::Error, BoolExpressionMethods, ExpressionMethods, OptionalExtension, QueryDsl,
    RunQueryDsl,
};
use serde::{Deserialize, Serialize};

#[derive(AsChangeset, Clone, Deserialize, Insertable, Queryable, Serialize)]
#[table_name = "warehouse_package"]
pub struct Package {
    pub id: String,
    pub creation_date: NaiveDateTime,
    pub modification_date: NaiveDateTime,
    pub name: String,
    pub version: String,
    pub description: String,
    pub url: String,
    pub build_date: NaiveDateTime,
    pub compressed_size: i64,
    pub installed_size: i64,
    pub architecture: String,
    pub license: String,
    pub extension: String,
    pub repository_id: String,
    pub maintainer_id: String,
}

impl Package {
    pub fn count(connection: &Connection) -> Result<i64, Error> {
        Ok(warehouse_package::table
            .select(dsl::count_star())
            .first(connection)?)
    }

    pub fn create(&self, connection: &Connection) -> Result<(), Error> {
        dsl::insert_into(warehouse_package::table)
            .values(self)
            .execute(connection)?;
        Ok(())
    }

    pub fn delete(&self, connection: &Connection) -> Result<(), Error> {
        dsl::delete(warehouse_package::table.filter(warehouse_package::id.eq(&self.id)))
            .execute(connection)?;
        Ok(())
    }

    pub fn delete_dependencies(&self, connection: &Connection) -> Result<(), Error> {
        dsl::delete(
            warehouse_package_dependency::table
                .filter(warehouse_package_dependency::package_id.eq(&self.id)),
        )
        .execute(connection)?;
        Ok(())
    }

    pub fn delete_files(&self, connection: &Connection) -> Result<(), Error> {
        dsl::delete(
            warehouse_package_file::table.filter(warehouse_package_file::package_id.eq(&self.id)),
        )
        .execute(connection)?;
        Ok(())
    }

    pub fn delete_versions(&self, connection: &Connection) -> Result<(), Error> {
        dsl::delete(
            warehouse_package_version::table
                .filter(warehouse_package_version::package_id.eq(&self.id)),
        )
        .execute(connection)?;
        Ok(())
    }

    pub fn find_by_name_repository_and_architecture(
        connection: &Connection,
        name: &str,
        repository: &str,
        architecture: &str,
    ) -> Result<Option<(Package, Repository, User)>, Error> {
        Ok(warehouse_package::table
            .inner_join(warehouse_repository::table)
            .inner_join(warehouse_user::table)
            .select((
                warehouse_package::all_columns,
                warehouse_repository::all_columns,
                warehouse_user::all_columns,
            ))
            .filter(
                warehouse_package::name
                    .eq(name)
                    .and(warehouse_package::architecture.eq(architecture))
                    .and(warehouse_repository::name.eq(repository)),
            )
            .first(connection)
            .optional()?)
    }

    pub fn list(
        connection: &Connection,
        offset: i64,
        limit: i64,
    ) -> Result<Vec<(Package, Repository, User)>, Error> {
        Ok(warehouse_package::table
            .inner_join(warehouse_repository::table)
            .inner_join(warehouse_user::table)
            .select((
                warehouse_package::all_columns,
                warehouse_repository::all_columns,
                warehouse_user::all_columns,
            ))
            .order_by(warehouse_package::name)
            .offset(offset)
            .limit(limit)
            .load(connection)?)
    }

    pub fn list_dependencies(&self, connection: &Connection) -> Result<Vec<Dependency>, Error> {
        Ok(warehouse_package_dependency::table
            .select(warehouse_package_dependency::all_columns)
            .filter(warehouse_package_dependency::package_id.eq(&self.id))
            .order_by(warehouse_package_dependency::name)
            .load(connection)?)
    }

    pub fn list_files(&self, connection: &Connection) -> Result<Vec<File>, Error> {
        Ok(warehouse_package_file::table
            .select(warehouse_package_file::all_columns)
            .filter(warehouse_package_file::package_id.eq(&self.id))
            .order_by(warehouse_package_file::name)
            .load(connection)?)
    }

    pub fn list_versions(&self, connection: &Connection) -> Result<Vec<(Version, User)>, Error> {
        Ok(warehouse_package_version::table
            .inner_join(warehouse_user::table)
            .select((
                warehouse_package_version::all_columns,
                warehouse_user::all_columns,
            ))
            .filter(warehouse_package_version::package_id.eq(&self.id))
            .order_by(warehouse_package_version::creation_date.desc())
            .load(connection)?)
    }

    pub fn update(&self, connection: &Connection) -> Result<(), Error> {
        dsl::update(warehouse_package::table)
            .set(self)
            .filter(warehouse_package::id.eq(&self.id))
            .execute(connection)?;
        Ok(())
    }
}

#[derive(Clone, Deserialize, Insertable, Queryable, Serialize)]
#[table_name = "warehouse_package_dependency"]
pub struct Dependency {
    pub id: String,
    pub name: String,
    pub package_id: String,
}

impl Dependency {
    pub fn create(&self, connection: &Connection) -> Result<(), Error> {
        dsl::insert_into(warehouse_package_dependency::table)
            .values(self)
            .execute(connection)?;
        Ok(())
    }
}

#[derive(Clone, Deserialize, Insertable, Queryable, Serialize)]
#[table_name = "warehouse_package_file"]
pub struct File {
    pub id: String,
    pub name: String,
    pub size: i64,
    pub package_id: String,
}

impl File {
    pub fn create(&self, connection: &Connection) -> Result<(), Error> {
        dsl::insert_into(warehouse_package_file::table)
            .values(self)
            .execute(connection)?;
        Ok(())
    }
}

#[derive(Clone, Deserialize, Insertable, Queryable, Serialize)]
#[table_name = "warehouse_package_version"]
pub struct Version {
    pub id: String,
    pub creation_date: NaiveDateTime,
    pub version: String,
    pub maintainer_id: String,
    pub package_id: String,
}

impl Version {
    pub fn create(&self, connection: &Connection) -> Result<(), Error> {
        dsl::insert_into(warehouse_package_version::table)
            .values(self)
            .execute(connection)?;
        Ok(())
    }
}
