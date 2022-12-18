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
use serde::{Deserialize, Serialize};
use url::Url;

use crate::ctx::Ctx;
use crate::db::AddGiteaInstance;
use crate::errors::ServiceResult;

impl Ctx {
    pub async fn init_gitea_instance(&self, info: &AddGiteaInstance) -> ServiceResult<()> {
        let mut url = info.url.clone();
        url.set_path("/.well-known/openid-configuration");
        let res: OIDCConfiguration = self
            .client
            .get(url)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        self.db.new_gitea_instance(&info).await?;
        self.db
            .new_gitea_oidc_configuration(&info.url, &res)
            .await?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct OIDCConfiguration {
    pub authorization_endpoint: Url,
    pub token_endpoint: Url,
    pub userinfo_endpoint: Url,
    pub introspection_endpoint: Url,
}
