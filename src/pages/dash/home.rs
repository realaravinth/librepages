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
use std::cell::RefCell;

use actix_identity::Identity;
use actix_web::http::header::ContentType;
use serde::{Deserialize, Serialize};
use tera::Context;

use super::get_auth_middleware;
use crate::ctx::api::v1::auth::Login as LoginPayload;
use crate::db::Site;
use crate::errors::ServiceResult;
use crate::pages::errors::*;
use crate::settings::Settings;
use crate::AppCtx;

use crate::pages::errors::*;

use super::TemplateSiteEvent;

pub use super::*;

pub const DASH_HOME: TemplateFile = TemplateFile::new("dash_home", "pages/dash/index.html");

pub struct Home {
    ctx: RefCell<Context>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct TemplateSite {
    site: Site,
    last_update: Option<TemplateSiteEvent>,
}

impl CtxError for Home {
    fn with_error(&self, e: &ReadableError) -> String {
        self.ctx.borrow_mut().insert(ERROR_KEY, e);
        self.render()
    }
}

impl Home {
    pub fn new(settings: &Settings, sites: Option<&[TemplateSite]>) -> Self {
        let ctx = RefCell::new(context(settings));
        if let Some(sites) = sites {
            ctx.borrow_mut().insert(PAYLOAD_KEY, sites);
        }
        Self { ctx }
    }

    pub fn render(&self) -> String {
        TEMPLATES
            .render(DASH_HOME.name, &self.ctx.borrow())
            .unwrap()
    }
}

async fn get_site_data(ctx: &AppCtx, id: &Identity) -> ServiceResult<Vec<TemplateSite>> {
    let db_sites = ctx.db.list_all_sites(&id.identity().unwrap()).await?;
    let mut sites = Vec::with_capacity(db_sites.len());
    for site in db_sites {
        // TODO: impl method on DB to get latest "update" event
        let mut events = ctx.db.list_all_site_events(&site.hostname).await?;
        let last_update = if let Some(event) = events.pop() {
            Some(event.into())
        } else {
            None
        };

        sites.push(TemplateSite { site, last_update });
    }
    Ok(sites)
}

#[actix_web_codegen_const_routes::get(path = "PAGES.dash.home", wrap = "get_auth_middleware()")]
#[tracing::instrument(name = "Dashboard homepage", skip(ctx, id))]
pub async fn get_home(ctx: AppCtx, id: Identity) -> PageResult<impl Responder, Home> {
    let sites = get_site_data(&ctx, &id)
        .await
        .map_err(|e| PageError::new(Home::new(&ctx.settings, None), e))?;
    let home = Home::new(&ctx.settings, Some(&sites)).render();
    let html = ContentType::html();
    Ok(HttpResponse::Ok().content_type(html).body(home))
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(get_home);
}
