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

use actix_web::http::header::ContentType;
use tera::Context;

use crate::db::AddGiteaInstance;
use crate::pages::errors::*;
use crate::settings::Settings;
use crate::AppCtx;

pub use super::*;

pub struct GiteaAddInstanceTemplate {
    ctx: RefCell<Context>,
}

pub const GITEA_SEARCH_INSTANCE: TemplateFile =
    TemplateFile::new("gitea_add_instance", "pages/auth/gitea/add.html");

impl CtxError for GiteaAddInstanceTemplate {
    fn with_error(&self, e: &ReadableError) -> String {
        self.ctx.borrow_mut().insert(ERROR_KEY, e);
        self.render()
    }
}

impl GiteaAddInstanceTemplate {
    pub fn new(settings: &Settings, payload: Option<&AddGiteaInstance>) -> Self {
        let ctx = RefCell::new(context(settings));
        if let Some(payload) = payload {
            ctx.borrow_mut().insert(PAYLOAD_KEY, payload);
        }
        Self { ctx }
    }

    pub fn render(&self) -> String {
        TEMPLATES
            .render(GITEA_SEARCH_INSTANCE.name, &self.ctx.borrow())
            .unwrap()
    }

    pub fn page(s: &Settings) -> String {
        let p = Self::new(s, None);
        p.render()
    }
}

#[actix_web_codegen_const_routes::get(path = "PAGES.auth.gitea.add")]
#[tracing::instrument(name = "Serve add Gitea instance page", skip(ctx))]
pub async fn get_gitea_add_instance(ctx: AppCtx) -> impl Responder {
    let login = GiteaAddInstanceTemplate::page(&ctx.settings);
    let html = ContentType::html();
    HttpResponse::Ok().content_type(html).body(login)
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(get_gitea_add_instance);
    cfg.service(post_gitea_add_instance);
}

#[actix_web_codegen_const_routes::post(path = "PAGES.auth.gitea.add")]
#[tracing::instrument(name = "Submit new Gitea instance", skip(payload, ctx))]
pub async fn post_gitea_add_instance(
    payload: web::Form<AddGiteaInstance>,
    ctx: AppCtx,
) -> PageResult<impl Responder, GiteaAddInstanceTemplate> {
    let payload = payload.into_inner();
    ctx.init_gitea_instance(&payload).await.map_err(|e| {
        PageError::new(
            GiteaAddInstanceTemplate::new(&ctx.settings, Some(&payload)),
            e,
        )
    })?;
    Ok(HttpResponse::Found()
        .insert_header((http::header::LOCATION, PAGES.dash.home))
        .finish())
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::GiteaAddInstanceTemplate;
    use crate::db::AddGiteaInstance;
    use crate::errors::*;
    use crate::pages::errors::*;
    use crate::settings::Settings;

    #[test]
    fn gitea_add_instnace_page_renders() {
        let settings = Settings::new().unwrap();
        GiteaAddInstanceTemplate::page(&settings);
        let payload = AddGiteaInstance {
            client_id: "foo".into(),
            client_secret: "foo".into(),
            url: Url::parse("https://example.org").unwrap(),
        };
        let page = GiteaAddInstanceTemplate::new(&settings, Some(&payload));
        page.with_error(&ReadableError::new(&ServiceError::WrongPassword));
        page.render();
    }
}
