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

pub struct Login {
    ctx: RefCell<Context>,
}

pub const LOGIN: TemplateFile = TemplateFile::new("login", "pages/auth/login.html");

impl CtxError for Login {
    fn with_error(&self, e: &ReadableError) -> String {
        self.ctx.borrow_mut().insert(ERROR_KEY, e);
        self.render()
    }
}

impl Login {
    pub fn new(settings: &Settings, payload: Option<&LoginPayload>) -> Self {
        let ctx = RefCell::new(context(settings));
        if let Some(payload) = payload {
            ctx.borrow_mut().insert(PAYLOAD_KEY, payload);
        }
        Self { ctx }
    }

    pub fn render(&self) -> String {
        TEMPLATES.render(LOGIN.name, &self.ctx.borrow()).unwrap()
    }

    pub fn page(s: &Settings) -> String {
        let p = Self::new(s, None);
        p.render()
    }
}

#[actix_web_codegen_const_routes::get(path = "PAGES.auth.login")]
pub async fn get_login(ctx: AppCtx) -> impl Responder {
    let login = Login::page(&ctx.settings);
    let html = ContentType::html();
    HttpResponse::Ok().content_type(html).body(login)
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(get_login);
    cfg.service(login_submit);
}

#[actix_web_codegen_const_routes::post(path = "PAGES.auth.login")]
pub async fn login_submit(
    id: Identity,
    payload: web::Form<LoginPayload>,
    query: web::Query<RedirectQuery>,
    ctx: AppCtx,
) -> PageResult<impl Responder, Login> {
    let username = ctx
        .login(&payload)
        .await
        .map_err(|e| PageError::new(Login::new(&ctx.settings, Some(&payload)), e))?;
    id.remember(username);
    let query = query.into_inner();
    if let Some(redirect_to) = query.redirect_to {
        Ok(HttpResponse::Found()
            .insert_header((http::header::LOCATION, redirect_to))
            .finish())
    } else {
        Ok(HttpResponse::Found()
            .insert_header((http::header::LOCATION, PAGES.home))
            .finish())
    }
}

#[cfg(test)]
mod tests {
    use super::Login;
    use super::LoginPayload;
    use crate::errors::*;
    use crate::pages::errors::*;
    use crate::settings::Settings;

    #[test]
    fn register_page_renders() {
        let settings = Settings::new().unwrap();
        Login::page(&settings);
        let payload = LoginPayload {
            login: "foo".into(),
            password: "foo".into(),
        };
        let page = Login::new(&settings, Some(&payload));
        page.with_error(&ReadableError::new(&ServiceError::WrongPassword));
        page.render();
    }
}
