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
use actix_web::web;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::ctx::Ctx;
use crate::db;
use crate::db::Site;
use crate::errors::*;
use crate::page::Page;
use crate::page_config;
use crate::settings::Settings;
use crate::subdomains::get_random_subdomain;
use crate::utils::get_random;
use crate::utils::get_website_path;

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
/// Data required to add site
pub struct AddSite {
    pub repo_url: String,
    pub branch: String,
    pub owner: String,
}

impl AddSite {
    fn to_site(self, s: &Settings) -> Site {
        let site_secret = get_random(32);
        let hostname = get_random_subdomain(s);
        let pub_id = Uuid::new_v4();
        Site {
            site_secret,
            repo_url: self.repo_url,
            branch: self.branch,
            hostname,
            owner: self.owner,
            pub_id,
        }
    }
}

impl Ctx {
    pub async fn add_site(&self, site: AddSite) -> ServiceResult<Page> {
        let db_site = site.to_site(&self.settings);
        self.db.add_site(&db_site).await?;
        let page = Page::from_site(&self.settings, db_site);
        page.update(&page.branch)?;
        self.db
            .log_event(&page.domain, &db::EVENT_TYPE_CREATE)
            .await?;
        self.conductor.new_site(page.clone()).await?;

        if let Some(config) = page_config::load(&page.path, &page.branch) {
            self.conductor.tx_config(config).await?;
            unimplemented!("Parse and store custom domains in DB");
        }
        Ok(page)
    }

    pub async fn update_site(&self, secret: &str, branch: Option<String>) -> ServiceResult<Uuid> {
        if let Ok(db_site) = self.db.get_site_from_secret(secret).await {
            let page = Page::from_site(&self.settings, db_site);
            let (tx, rx) = oneshot::channel();
            {
                let page = page.clone();
                web::block(move || {
                    if let Some(branch) = branch {
                        tx.send(page.update(&branch)).unwrap();
                    } else {
                        tx.send(page.update(&page.branch)).unwrap();
                    }
                })
                .await
                .unwrap();
            }
            rx.await.unwrap()?;
            if let Some(config) = page_config::load(&page.path, &page.branch) {
                self.conductor.tx_config(config).await?;
                unimplemented!("Parse and store custom domains in DB");
            }
            self.db
                .log_event(&page.domain, &db::EVENT_TYPE_UPDATE)
                .await
        } else {
            Err(ServiceError::WebsiteNotFound)
        }
    }

    pub async fn delete_site(&self, owner: String, site_id: Uuid) -> ServiceResult<()> {
        if let Ok(db_site) = self.db.get_site_from_pub_id(site_id, owner).await {
            let path = get_website_path(&self.settings, &db_site.hostname);

            self.db
                .log_event(&db_site.hostname, &db::EVENT_TYPE_DELETE)
                .await?;

            fs::remove_dir_all(&path).await?;
            self.db
                .delete_site(&db_site.owner, &db_site.hostname)
                .await?;
            self.conductor.delete_site(db_site.hostname).await?;
            Ok(())
        } else {
            Err(ServiceError::WebsiteNotFound)
        }
    }
}
