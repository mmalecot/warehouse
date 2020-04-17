CREATE TABLE warehouse_user
(
    id            VARCHAR(36)        NOT NULL,
    creation_date TIMESTAMP          NOT NULL,
    name          VARCHAR(20) UNIQUE NOT NULL,
    email         TEXT UNIQUE        NOT NULL,
    password      TEXT               NOT NULL,
    admin         BOOLEAN            NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE warehouse_repository
(
    id        VARCHAR(36)        NOT NULL,
    name      VARCHAR(20) UNIQUE NOT NULL,
    extension TEXT               NOT NULL,
    PRIMARY KEY (id)
);

INSERT INTO warehouse_repository
VALUES ('50f13e5d-81e3-4099-828d-47f3c4f30ffd', 'main', 'db.tar.zst');

CREATE TABLE warehouse_package
(
    id                VARCHAR(36) NOT NULL,
    creation_date     TIMESTAMP   NOT NULL,
    modification_date TIMESTAMP   NOT NULL,
    name              TEXT        NOT NULL,
    version           TEXT        NOT NULL,
    description       TEXT        NOT NULL,
    url               TEXT        NOT NULL,
    build_date        TIMESTAMP   NOT NULL,
    compressed_size   BIGINT      NOT NULL,
    installed_size    BIGINT      NOT NULL,
    architecture      TEXT        NOT NULL,
    license           TEXT        NOT NULL,
    extension         TEXT        NOT NULL,
    repository_id     VARCHAR(36) NOT NULL,
    maintainer_id     VARCHAR(36) NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (repository_id) REFERENCES warehouse_repository (id),
    FOREIGN KEY (maintainer_id) REFERENCES warehouse_user (id)
);

CREATE UNIQUE INDEX warehouse_package_idx1 ON warehouse_package (name, architecture, repository_id);

CREATE TABLE warehouse_package_file
(
    id         VARCHAR(36) NOT NULL,
    name       TEXT        NOT NULL,
    size       BIGINT      NOT NULL,
    package_id VARCHAR(36) NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (package_id) REFERENCES warehouse_package (id)
);

CREATE TABLE warehouse_package_dependency
(
    id         VARCHAR(36) NOT NULL,
    name       TEXT        NOT NULL,
    package_id VARCHAR(36) NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (package_id) REFERENCES warehouse_package (id)
);

CREATE TABLE warehouse_package_version
(
    id            VARCHAR(36) NOT NULL,
    creation_date TIMESTAMP   NOT NULL,
    version       TEXT        NOT NULL,
    maintainer_id VARCHAR(36) NOT NULL,
    package_id    VARCHAR(36) NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (maintainer_id) REFERENCES warehouse_user (id),
    FOREIGN KEY (package_id) REFERENCES warehouse_package (id)
);
