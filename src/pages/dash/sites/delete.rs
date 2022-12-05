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
use uuid::Uuid;

use super::get_auth_middleware;

use crate::ctx::api::v1::auth::{Login, Password};
use crate::db::Site;
use crate::pages::dash::TemplateSiteEvent;
use crate::pages::errors::*;
use crate::settings::Settings;
use crate::AppCtx;

pub use super::*;

pub const DASH_SITE_DELETE: TemplateFile =
    TemplateFile::new("dash_site_delete", "pages/dash/sites/delete.html");

const SHOW_DEPLOY_SECRET_KEY: &str = "show_deploy_secret";

pub struct Delete {
    ctx: RefCell<Context>,
}

impl CtxError for Delete {
    fn with_error(&self, e: &ReadableError) -> String {
        self.ctx.borrow_mut().insert(ERROR_KEY, e);
        self.render()
    }
}

impl Delete {
    pub fn new(settings: &Settings, payload: Option<TemplateSiteWithEvents>) -> Self {
        let ctx = RefCell::new(context(settings));
        if let Some(payload) = payload {
            ctx.borrow_mut().insert(PAYLOAD_KEY, &payload);
        }

        Self { ctx }
    }

    pub fn show_deploy_secret(&mut self) {
        self.ctx.borrow_mut().insert(SHOW_DEPLOY_SECRET_KEY, &true);
    }

    pub fn render(&self) -> String {
        TEMPLATES
            .render(DASH_SITE_DELETE.name, &self.ctx.borrow())
            .unwrap()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct TemplateSiteWithEvents {
    pub site: Site,
    pub delete: String,
    pub last_update: Option<TemplateSiteEvent>,
    pub events: Vec<TemplateSiteEvent>,
}

impl TemplateSiteWithEvents {
    pub fn new(
        site: Site,
        last_update: Option<TemplateSiteEvent>,
        events: Vec<TemplateSiteEvent>,
    ) -> Self {
        let delete = PAGES.dash.site.get_delete(site.pub_id);
        Self {
            site,
            last_update,
            delete,
            events,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct DeleteOptions {
    show_deploy_secret: Option<bool>,
}

#[actix_web_codegen_const_routes::get(
    path = "PAGES.dash.site.delete",
    wrap = "get_auth_middleware()"
)]
#[tracing::instrument(name = "Dashboard delete site webpage", skip(ctx, id))]
pub async fn get_delete_site(
    ctx: AppCtx,
    id: Identity,
    path: web::Path<Uuid>,
    query: web::Query<DeleteOptions>,
) -> PageResult<impl Responder, Delete> {
    let site_id = path.into_inner();
    let owner = id.identity().unwrap();

    let site = ctx
        .db
        .get_site_from_pub_id(site_id, owner)
        .await
        .map_err(|e| PageError::new(Delete::new(&ctx.settings, None), e))?;
    let last_update = ctx
        .db
        .get_latest_update_event(&site.hostname)
        .await
        .map_err(|e| PageError::new(Delete::new(&ctx.settings, None), e))?;

    let last_update = last_update.map(|e| e.into());

    let mut db_events = ctx
        .db
        .list_all_site_events(&site.hostname)
        .await
        .map_err(|e| PageError::new(Delete::new(&ctx.settings, None), e))?;

    let mut events = Vec::with_capacity(db_events.len());
    for e in db_events.drain(0..) {
        events.push(e.into());
    }

    let payload = TemplateSiteWithEvents::new(site, last_update, events);
    let mut page = Delete::new(&ctx.settings, Some(payload));
    if let Some(true) = query.show_deploy_secret {
        page.show_deploy_secret();
    }
    let add = page.render();
    let html = ContentType::html();
    Ok(HttpResponse::Ok().content_type(html).body(add))
}

#[actix_web_codegen_const_routes::post(
    path = "PAGES.dash.site.delete",
    wrap = "get_auth_middleware()"
)]
#[tracing::instrument(name = "Delete site from webpage", skip(ctx, id))]
pub async fn post_delete_site(
    ctx: AppCtx,
    id: Identity,
    path: web::Path<Uuid>,
    payload: web::Form<Password>,
) -> PageResult<impl Responder, Delete> {
    let site_id = path.into_inner();
    let owner = id.identity().unwrap();

    let payload = payload.into_inner();
    let msg = Login {
        login: owner,
        password: payload.password,
    };
    ctx.login(&msg)
        .await
        .map_err(|e| PageError::new(Delete::new(&ctx.settings, None), e))?;

    ctx.delete_site(msg.login, site_id)
        .await
        .map_err(|e| PageError::new(Delete::new(&ctx.settings, None), e))?;

    Ok(HttpResponse::Found()
        .append_header((http::header::LOCATION, PAGES.dash.home))
        .finish())
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(get_delete_site);
    cfg.service(post_delete_site);
}
