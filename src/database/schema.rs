table! {
    warehouse_package (id) {
        id -> Text,
        creation_date -> Timestamp,
        modification_date -> Timestamp,
        name -> Text,
        version -> Text,
        description -> Text,
        url -> Text,
        build_date -> Timestamp,
        compressed_size -> BigInt,
        installed_size -> BigInt,
        architecture -> Text,
        license -> Text,
        extension -> Text,
        repository_id -> Text,
        maintainer_id -> Text,
    }
}

table! {
    warehouse_package_dependency (id) {
        id -> Text,
        name -> Text,
        package_id -> Text,
    }
}

table! {
    warehouse_package_file (id) {
        id -> Text,
        name -> Text,
        size -> BigInt,
        package_id -> Text,
    }
}

table! {
    warehouse_package_version (id) {
        id -> Text,
        creation_date -> Timestamp,
        version -> Text,
        maintainer_id -> Text,
        package_id -> Text,
    }
}

table! {
    warehouse_repository (id) {
        id -> Text,
        name -> Text,
        extension -> Text,
    }
}

table! {
    warehouse_user (id) {
        id -> Text,
        creation_date -> Timestamp,
        name -> Text,
        email -> Text,
        password -> Text,
        admin -> Bool,
    }
}

joinable!(warehouse_package -> warehouse_repository (repository_id));
joinable!(warehouse_package -> warehouse_user (maintainer_id));
joinable!(warehouse_package_dependency -> warehouse_package (package_id));
joinable!(warehouse_package_file -> warehouse_package (package_id));
joinable!(warehouse_package_version -> warehouse_user (maintainer_id));
joinable!(warehouse_package_version -> warehouse_package (package_id));

allow_tables_to_appear_in_same_query!(
    warehouse_package,
    warehouse_repository,
    warehouse_user,
    warehouse_package_version
);
