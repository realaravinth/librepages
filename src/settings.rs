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

use config::{Config, Environment, File};
#[cfg(not(test))]
use log::{error, warn};

#[cfg(test)]
use std::{println as warn, println as error};

use serde::Deserialize;
use url::Url;

use crate::errors::*;
use crate::page::Page;

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    pub port: u32,
    pub ip: String,
    pub workers: Option<usize>,
    pub domain: String,
}

impl Server {
    #[cfg(not(tarpaulin_include))]
    pub fn get_ip(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub server: Server,
    pub source_code: String,
    pub pages: Vec<Arc<Page>>,
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

        let mut settings = s.build()?.try_deserialize::<Settings>()?;
        settings.check_url();
        match env::var("PORT") {
            Ok(val) => {
                settings.server.port = val.parse().unwrap();
            }
            Err(e) => warn!("couldn't interpret PORT: {}", e),
        }

        settings.init();

        Ok(settings)
    }

    pub fn init(&self) {
        for (index, page) in self.pages.iter().enumerate() {
            Url::parse(&page.repo).unwrap();
            let path = Path::new(&page.path);
            if path.exists() && path.is_file() {
                panic!("Path is a file, should be a directory: {:?}", page);
            }

            if !path.exists() {
                std::fs::create_dir_all(&path).unwrap();
            }
            for (index2, page2) in self.pages.iter().enumerate() {
                if index2 == index {
                    continue;
                }
                if page.secret == page2.secret {
                    error!("{}", ServiceError::SecretTaken(page.clone(), page2.clone()));
                } else if page.repo == page2.repo {
                    error!(
                        "{}",
                        ServiceError::DuplicateRepositoryURL(page.clone(), page2.clone(),)
                    );
                } else if page.path == page2.path {
                    error!("{}", ServiceError::PathTaken(page.clone(), page2.clone()));
                }
            }
            if let Err(e) = page.update(&page.branch) {
                error!("{e}");
            }
        }
    }

    #[cfg(not(tarpaulin_include))]
    fn check_url(&self) {
        Url::parse(&self.source_code).expect("Please enter a URL for source_code in settings");
    }
}
