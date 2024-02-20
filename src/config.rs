use std::{str::FromStr, sync::OnceLock};

use crate::web::crypt::base64::b64u_decode;

pub fn config() -> &'static Config {
    static INSTANCE: OnceLock<Config> = OnceLock::new();

    INSTANCE.get_or_init(Config::load_from_env)
}

#[allow(non_snake_case)]
pub struct Config {
    pub DB_URL: String,
    pub IMAGES_DIR: String,
    pub LUST_URL: String,
    pub LUST_PROFILE_BUCKET: String,
    pub TOKEN_SECRET: Vec<u8>,
    pub PWD_KEY: Vec<u8>,
    pub TOKEN_DURATION: i64,
    pub PORT: i32,
    pub LUST_BUCKET: String,
}

impl Config {
    fn load_from_env() -> Config {
        Config {
            DB_URL: get_env("DATABASE_URL"),
            IMAGES_DIR: get_env("IMAGES_DIR"),
            LUST_URL: get_env("LUST_URL"),
            LUST_PROFILE_BUCKET: get_env("LUST_PROFILE_BUCKET"),
            TOKEN_SECRET: get_env_b64u_as_u8s("TOKEN_SECRET"),
            PWD_KEY: get_env_b64u_as_u8s("PWD_KEY"),
            TOKEN_DURATION: get_env_parse_or("TOKEN_DURATION", None),
            PORT: get_env_parse_or("PORT", Some(3000)),
            LUST_BUCKET: get_env("LUST_BUCKET"),
        }
    }
}

fn get_env(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("{} is not set", key))
}

fn get_env_parse_or<T>(name: &'static str, or: Option<T>) -> T
where
    T: FromStr,
    T::Err: std::fmt::Debug,
{
    let val = get_env(name);
    match or {
        Some(or) => val.parse::<T>().unwrap_or(or),
        None => val
            .parse::<T>()
            .unwrap_or_else(|_| panic!("{} is not a valid {}", val, name)),
    }
}

fn get_env_b64u_as_u8s(name: &'static str) -> Vec<u8> {
    b64u_decode(&get_env(name)).unwrap_or_else(|_| panic!("{} is not a valid b64u", name))
}
