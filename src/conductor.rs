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
use reqwest::Client;

use libconductor::EventType;
use libconfig::Config;
use tracing::info;

use crate::errors::ServiceResult;
use crate::{page::Page, settings::Settings};

#[derive(Clone)]
pub struct Conductor {
    client: Client,
    pub settings: Settings,
}

impl Conductor {
    pub fn new(settings: Settings) -> Self {
        Self {
            client: Client::new(),
            settings,
        }
    }
    async fn tx(&self, e: &EventType) -> ServiceResult<()> {
        for c in self.settings.conductors.iter() {
            info!("Tx event to {}", c.url);
            let mut event_url = c.url.clone();
            event_url.set_path("/api/v1/events/new");
            self.client
                .post(event_url)
                .basic_auth(&c.username, Some(&c.api_key))
                .json(e)
                .send()
                .await
                .unwrap();
        }
        Ok(())
    }

    pub async fn new_site(&self, page: Page) -> ServiceResult<()> {
        let msg = EventType::NewSite {
            hostname: page.domain,
            branch: page.branch,
            path: page.path,
        };
        self.tx(&msg).await
    }

    pub async fn tx_config(&self, config: Config) -> ServiceResult<()> {
        self.tx(&EventType::Config { data: config }).await
    }

    pub async fn delete_site(&self, hostname: String) -> ServiceResult<()> {
        self.tx(&EventType::DeleteSite { hostname }).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use uuid::Uuid;

    #[actix_rt::test]
    pub async fn test_conductor() {
        let settings = Settings::new().unwrap();
        let c = Conductor::new(settings.clone());
        c.delete_site("example.org".into()).await.unwrap();
        let page = Page {
            secret: "foo".into(),
            repo: "foo".into(),
            path: "foo".into(),
            branch: "foo".into(),
            domain: "foo".into(),
            pub_id: Uuid::new_v4(),
        };
        c.new_site(page).await.unwrap();
    }
}
