/*
 * Copyright (C) 2021  Aravinth Manivannan <realaravinth@batsense.net>
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

use config::{Config, ConfigError, Environment, File};
use log::warn;
use serde::Deserialize;
use url::Url;

use crate::page::Page;

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    pub port: u32,
    pub domain: String,
    pub ip: String,
    pub proxy_has_tls: bool,
    pub workers: Option<usize>,
}

impl Server {
    #[cfg(not(tarpaulin_include))]
    pub fn get_ip(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub debug: bool,
    //    pub database: Database,
    pub server: Server,
    pub source_code: String,
    pub pages: Vec<Page>,
}

#[cfg(not(tarpaulin_include))]
impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        // setting default values
        #[cfg(test)]
        s.set_default("database.pool", 2.to_string())
            .expect("Couldn't get the number of CPUs");

        const CURRENT_DIR: &str = "./config/default.toml";
        const ETC: &str = "/etc/static-pages/config.toml";

        if let Ok(path) = env::var("ATHENA_CONFIG") {
            s.merge(File::with_name(&path))?;
        } else if Path::new(CURRENT_DIR).exists() {
            // merging default config from file
            s.merge(File::with_name(CURRENT_DIR))?;
        } else if Path::new(ETC).exists() {
            s.merge(File::with_name(ETC))?;
        } else {
            log::warn!("configuration file not found");
        }

        s.merge(Environment::with_prefix("PAGES").separator("__"))?;

        check_url(&s);

        match env::var("PORT") {
            Ok(val) => {
                s.set("server.port", val).unwrap();
            }
            Err(e) => warn!("couldn't interpret PORT: {}", e),
        }

        let settings: Settings = s.try_into()?;

        for (index, page) in settings.pages.iter().enumerate() {
            Url::parse(&page.repo).unwrap();
            let path = Path::new(&page.path);
            if path.exists() && path.is_file() {
                panic!("Path is a file, should be a directory: {:?}", page);
            }

            if !path.exists() {
                std::fs::create_dir_all(&path).unwrap();
            }
            for (index2, page2) in settings.pages.iter().enumerate() {
                if index2 == index {
                    continue;
                }
                if page.secret == page2.secret || page.repo == page2.repo || page.path == page2.path
                {
                    panic!("duplicate page onfiguration {:?} and {:?}", page, page2);
                }
            }
            page.fetch_upstream(&page.branch);
        }

        Ok(settings)
    }
}

#[cfg(not(tarpaulin_include))]
fn check_url(s: &Config) {
    let url = s
        .get::<String>("source_code")
        .expect("Couldn't access source_code");

    Url::parse(&url).expect("Please enter a URL for source_code in settings");
}
