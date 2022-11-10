/*
 * Copyright (C) 2022  Aravinth Manivannan <realaravinth@batsense.net>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use std::env;
use std::path::Path;
use std::sync::Arc;

use config::{Config, ConfigError, Environment, File};
use derive_more::Display;
#[cfg(not(test))]
use log::{error, warn};

#[cfg(test)]
use std::{println as warn, println as error};

use serde::Deserialize;
use serde::Serialize;
use url::Url;

use crate::errors::*;
use crate::page::Page;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub port: u32,
    pub ip: String,
    pub workers: Option<usize>,
    pub cookie_secret: String,
    pub domain: String,
}

impl Server {
    #[cfg(not(tarpaulin_include))]
    pub fn get_ip(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

#[derive(Deserialize, Serialize, Display, Eq, PartialEq, Clone, Debug)]
#[serde(rename_all = "lowercase")]
pub enum DBType {
    #[display(fmt = "postgres")]
    Postgres,
    //    #[display(fmt = "maria")]
    //    Maria,
}

impl DBType {
    fn from_url(url: &Url) -> Result<Self, ConfigError> {
        match url.scheme() {
            //        "mysql" => Ok(Self::Maria),
            "postgres" => Ok(Self::Postgres),
            _ => Err(ConfigError::Message("Unknown database type".into())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    pub url: String,
    pub pool: u32,
    pub database_type: DBType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub allow_registration: bool,
    pub support_email: String,
    pub debug: bool,
    pub server: Server,
    pub source_code: String,
    pub database: Database,
    pub page: PageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageConfig {
    pub base_path: String,
}

#[cfg(not(tarpaulin_include))]
impl Settings {
    pub fn new() -> ServiceResult<Self> {
        let mut s = Config::builder();

        const CURRENT_DIR: &str = "./config/default.toml";
        const ETC: &str = "/etc/static-pages/config.toml";

        let mut read_file = false;

        if Path::new(ETC).exists() {
            s = s.add_source(File::with_name(ETC));
            read_file = true;
        }
        if Path::new(CURRENT_DIR).exists() {
            // merging default config from file
            s = s.add_source(File::with_name(CURRENT_DIR));
            read_file = true;
        }

        if let Ok(path) = env::var("PAGES_CONFIG") {
            s = s.add_source(File::with_name(&path));
            read_file = true;
        }

        if !read_file {
            warn!("configuration file not found");
        }

        s = s.add_source(Environment::with_prefix("PAGES").separator("__"));

        match env::var("PORT") {
            Ok(val) => {
                s = s.set_override("server.port", val).unwrap();
            }
            Err(e) => warn!("couldn't interpret PORT: {}", e),
        }

        if let Ok(val) = env::var("DATABASE_URL") {
            let url = Url::parse(&val).expect("couldn't parse Database URL");
            s = s.set_override("database.url", url.to_string()).unwrap();
            let database_type = DBType::from_url(&url).unwrap();
            s = s
                .set_override("database.database_type", database_type.to_string())
                .unwrap();
        }

        let intermediate_config = s.build_cloned().unwrap();

        s = s
            .set_override(
                "database.url",
                format!(
                    r"postgres://{}:{}@{}:{}/{}",
                    intermediate_config
                        .get::<String>("database.username")
                        .expect("Couldn't access database username"),
                    intermediate_config
                        .get::<String>("database.password")
                        .expect("Couldn't access database password"),
                    intermediate_config
                        .get::<String>("database.hostname")
                        .expect("Couldn't access database hostname"),
                    intermediate_config
                        .get::<String>("database.port")
                        .expect("Couldn't access database port"),
                    intermediate_config
                        .get::<String>("database.name")
                        .expect("Couldn't access database name")
                ),
            )
            .expect("Couldn't set database url");

        let settings = s.build()?.try_deserialize::<Settings>()?;
        settings.check_url();

        Ok(settings)
    }

    pub fn init(&self) {
        fn create_dir_util(path: &Path) {
            if path.exists() && path.is_file() {
                panic!("Path is a file, should be a directory: {:?}", path);
            }

            if !path.exists() {
                std::fs::create_dir_all(&path).unwrap();
            }
        }

        // create_dir_util(Path::new(&page.path));
        create_dir_util(Path::new(&self.page.base_path));

        //        for (index, page) in self.pages.iter().enumerate() {
        //            Url::parse(&page.repo).unwrap();
        //
        //            for (index2, page2) in self.pages.iter().enumerate() {
        //                if index2 == index {
        //                    continue;
        //                }
        //                if page.secret == page2.secret {
        //                    error!("{}", ServiceError::SecretTaken(page.clone(), page2.clone()));
        //                } else if page.repo == page2.repo {
        //                    error!(
        //                        "{}",
        //                        ServiceError::DuplicateRepositoryURL(page.clone(), page2.clone(),)
        //                    );
        //                } else if page.path == page2.path {
        //                    error!("{}", ServiceError::PathTaken(page.clone(), page2.clone()));
        //                }
        //            }
        //            if let Err(e) = page.update(&page.branch) {
        //                error!("{e}");
        //            }
        //        }
    }

    #[cfg(not(tarpaulin_include))]
    fn check_url(&self) {
        Url::parse(&self.source_code).expect("Please enter a URL for source_code in settings");
    }
}
