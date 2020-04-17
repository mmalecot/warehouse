use crate::utils::path;
use log::LevelFilter;
use rand::Rng;
use std::{
    env,
    net::{IpAddr, Ipv6Addr},
};

macro_rules! config {
    {
        $($config:ident: $config_type:ident {
            $($field:ident: $field_type:ty => $field_default:expr,)*
        },)*
    } => {
        use {
            crate::core::error::LoadConfigError,
            serde::{Deserialize, Serialize},
            std::{fs, str::FromStr},
        };

        #[derive(Clone, Debug, Default, Deserialize, Serialize)]
        #[serde(default)]
        pub struct Config {
            $(
                pub $config: $config_type,
            )*
        }

        impl Config {
            pub fn load() -> Result<Config, LoadConfigError> {
                let path = path::config_file();
                let mut config = if path.exists() {
                    toml::from_str(&fs::read_to_string(path)?)?
                } else {
                    Config::default()
                };
                $(
                    $(
                        if let Ok(var) = env::var(concat!(
                            std::env!("CARGO_PKG_NAME"),
                            "_",
                            stringify!($config),
                            "_",
                            stringify!($field)
                        ).to_uppercase()) {
                            config.$config.$field = <$field_type>::from_str(&var)?;
                        }
                    )*
                )*
                Ok(config)
            }
        }

        $(
            #[derive(Clone, Debug, Deserialize, Serialize)]
            #[serde(default)]
            pub struct $config_type {
                $(
                    pub $field: $field_type,
                )*
            }

            impl Default for $config_type {
                fn default() -> $config_type {
                    $config_type {
                        $(
                            $field: $field_default.into(),
                        )*
                    }
                }
            }
        )*
    };
}

config! {
    database: DatabaseConfig {
        pool_connection_timeout: u64 => 30u64,
        pool_idle_timeout: u64 => 600u64,
        pool_max_lifetime: u64 => 1800u64,
        pool_max_size: u32 => 10u32,
        pool_min_idle: u32 => 0u32,
        url: String => {
            if cfg!(feature = "mysql") {
                format!(
                    "mysql://{username}:{password}@[{server}]:{port}/{database}",
                    username = env!("CARGO_PKG_NAME"),
                    password = env!("CARGO_PKG_NAME"),
                    server = Ipv6Addr::LOCALHOST,
                    port = 3306,
                    database = env!("CARGO_PKG_NAME")
                )
            } else if cfg!(feature = "postgres") {
                format!(
                    "postgres://{username}:{password}@[{server}]:{port}/{database}",
                    username = env!("CARGO_PKG_NAME"),
                    password = env!("CARGO_PKG_NAME"),
                    server = Ipv6Addr::LOCALHOST,
                    port = 5432,
                    database = env!("CARGO_PKG_NAME")
                )
            } else {
                format!("{directory}/{filename}",
                    directory = path::data_dir().display(),
                    filename = concat!(env!("CARGO_PKG_NAME"), ".db")
                )
            }
        },
    },
    logger: LoggerConfig {
        access_format: String => r#"%a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T"#,
        file_dispatch: bool => false,
        level: LevelFilter => LevelFilter::Info,
        time_format: String => "%Y-%m-%d %H:%M:%S%.3f",
    },
    server: ServerConfig {
        ip_address: IpAddr => Ipv6Addr::UNSPECIFIED,
        port: u16 => 8080u16,
        upload_limit: usize => 268_435_456usize,
        workers: usize => num_cpus::get(),
    },
    session: SessionConfig {
        cookie_name: String => concat!(env!("CARGO_PKG_NAME"), "_auth"),
        cookie_secure: bool => false,
        secret_key: String => base64::encode(&rand::thread_rng().gen::<[u8; 32]>()),
    },
    ui: UIConfig {
        paging_num: u32 => 10u32,
        primary_color: String => "#484a90",
        primary_dark_color: String => "#2f3177",
    },
}
