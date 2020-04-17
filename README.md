# Warehouse

![Platform](https://img.shields.io/badge/platform-linux-green.svg?logo=linux&logoColor=white)
![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.42+-blueviolet.svg?logo=rust)

Arch Linux repository manager.

## References

* [Learn Rust](https://www.rust-lang.org/learn)
* [Actix documentation](https://actix.rs/docs/)
* [Tera documentation](https://tera.netlify.com/docs/)
* [Diesel guides](https://diesel.rs/guides/)
* [Bootstrap website](https://getbootstrap.com/)
* [Font Awesome website](https://fontawesome.com/)
* [jQuery API](https://api.jquery.com/)
* [MySQL documentation](https://dev.mysql.com/doc/)
* [PostgreSQL manuals](https://www.postgresql.org/docs/manuals/)
* [SQLite documentation](https://www.sqlite.org/docs.html)
* [Docker documentation](https://docs.docker.com/)
* [Docker Compose documentation](https://docs.docker.com/compose/)

## Configuration

If the `run_in_place` feature is enabled, any changes to the **Warehouse** configuration file should be made in `config/application.toml`.
Without this feature, this will be found at `/etc/warehouse/application.toml`.

It can also be done through **environment variables**.

Note: A full restart is required for **Warehouse** configuration changes to take effect.

### Database (`database`):
 * `pool_connection_timeout`: **30**: Connection timeout used by the pool in seconds.
 * `pool_idle_timeout`: **600**: Idle timeout used by the pool in seconds.
 * `pool_max_lifetime`: **1800**: Maximum lifetime of connections in the pool.
 * `pool_max_size`: **10**: Maximum number of connections managed by the pool.
 * `pool_min_idle`: **0**: Minimum idle connection count maintained by the pool.
 * `url`: **\<depends on database type\>**: Database connection URL or file path (SQLite).
 
### Logger (`logger`):
 * `access_format`: **%a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T**: HTTP access format.
 * `file_dispatch`: **false**: Dispatch logs in `log/warehouse.log`.
 * `level`: **Info**: Logging level. Can be `Error`, `Warn`, `Info`, `Debug` or `Trace`.
 * `time_format`: **%Y-%m-%d %H:%M:%S%.3f**: Date and time format (`strftime` formatting syntax).

### Server (`server`):
 * `ip_address`: **[::]**: HTTP listen address.
 * `port`: **8080**: HTTP listen port.
 * `upload_limit`: **268435456**: Maximum file size in bytes.
 * `workers`: **\<number of available logical CPU\>**: Number of workers.
 
### Session (`session`):
 * `cookie_name`: **warehouse_auth**: The name of the cookie used for the session ID.
 * `cookie_secure`: **false**: Enable this to force using HTTPS for all session access.
 * `secret_key`: **\<random\>**: Base64 encoded secret key.

### UI (`ui`):
 * `paging_num`: **10**: Number of entries that are shown in one page.
 * `primary_color`: **#484a90**: Primary color.
 * `primary_dark_color`: **#2f3177**: Primary dark color.

## Database

**Warehouse** works with either **MySQL**, **PostgreSQL** or **SQLite** database.

You can use **Docker Compose** to create a development database using:

```
$ docker-compose -f docker/dev-postgres/docker-compose.yml up -d
```

Or

```
$ docker-compose -f docker/dev-mysql/docker-compose.yml up -d
```

## License

This project is licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT) at your option.
