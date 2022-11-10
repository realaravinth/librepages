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
use tokio::sync::oneshot;

use crate::ctx::Ctx;
use crate::db::Site;
use crate::errors::*;
use crate::page::Page;
use crate::utils::get_random;

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
/// Data required to add site
pub struct AddSite {
    pub repo_url: String,
    pub branch: String,
    pub hostname: String,
    pub owner: String,
}

impl AddSite {
    fn to_site(self) -> Site {
        let site_secret = get_random(32);
        Site {
            site_secret,
            repo_url: self.repo_url,
            branch: self.branch,
            hostname: self.hostname,
            owner: self.owner,
        }
    }
}

impl Ctx {
    pub async fn add_site(&self, site: AddSite) -> ServiceResult<()> {
        let db_site = site.to_site();
        self.db.add_site(&db_site).await?;
        let page = Page::from_site(&self.settings, db_site);
        page.update(&page.branch)?;
        Ok(())
    }

    pub async fn update_site(&self, secret: &str, branch: Option<String>) -> ServiceResult<()> {
        if let Ok(db_site) = self.db.get_site_from_secret(secret).await {
            let page = Page::from_site(&self.settings, db_site);
            let (tx, rx) = oneshot::channel();
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
            rx.await.unwrap()?;
            Ok(())
        } else {
            Err(ServiceError::WebsiteNotFound)
        }
    }
}