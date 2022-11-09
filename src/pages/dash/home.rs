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
use tera::Context;

use crate::api::v1::RedirectQuery;
use crate::ctx::api::v1::auth::Login as LoginPayload;
use crate::pages::errors::*;
use crate::settings::Settings;
use crate::AppCtx;

pub use super::*;

pub const DASH_HOME: TemplateFile = TemplateFile::new("dash_home", "pages/dash/index.html");

pub struct Home {
    ctx: RefCell<Context>,
}

impl CtxError for Home {
    fn with_error(&self, e: &ReadableError) -> String {
        self.ctx.borrow_mut().insert(ERROR_KEY, e);
        self.render()
    }
}

impl Home {
    pub fn new(settings: &Settings, payload: Option<&LoginPayload>) -> Self {
        let ctx = RefCell::new(context(settings));
        if let Some(payload) = payload {
            ctx.borrow_mut().insert(PAYLOAD_KEY, payload);
        }
        Self { ctx }
    }

    pub fn render(&self) -> String {
        TEMPLATES
            .render(DASH_HOME.name, &self.ctx.borrow())
            .unwrap()
    }

    pub fn page(s: &Settings) -> String {
        let p = Self::new(s, None);
        p.render()
    }
}

#[actix_web_codegen_const_routes::get(path = "PAGES.dash.home")]
pub async fn get_home(ctx: AppCtx) -> impl Responder {
    let home = Home::page(&ctx.settings);
    let html = ContentType::html();
    HttpResponse::Ok().content_type(html).body(home)
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(get_home);
}
