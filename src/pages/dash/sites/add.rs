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
use crate::ctx::api::v1::pages::AddSite;
use crate::pages::errors::*;
use crate::settings::Settings;
use crate::AppCtx;

pub use super::*;

pub const DASH_SITE_ADD: TemplateFile =
    TemplateFile::new("dash_site_add", "pages/dash/sites/add.html");

pub struct Add {
    ctx: RefCell<Context>,
}

impl CtxError for Add {
    fn with_error(&self, e: &ReadableError) -> String {
        self.ctx.borrow_mut().insert(ERROR_KEY, e);
        self.render()
    }
}

impl Add {
    pub fn new(settings: &Settings) -> Self {
        let ctx = RefCell::new(context(settings));
        Self { ctx }
    }

    pub fn render(&self) -> String {
        TEMPLATES
            .render(DASH_SITE_ADD.name, &self.ctx.borrow())
            .unwrap()
    }
}

#[actix_web_codegen_const_routes::get(path = "PAGES.dash.site.add", wrap = "get_auth_middleware()")]
#[tracing::instrument(name = "Dashboard add site webpage", skip(ctx))]
pub async fn get_add_site(ctx: AppCtx) -> PageResult<impl Responder, Add> {
    let add = Add::new(&ctx.settings).render();
    let html = ContentType::html();
    Ok(HttpResponse::Ok().content_type(html).body(add))
}

#[derive(Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
/// Data required to add site
pub struct TemplateAddSite {
    pub repo_url: String,
    pub branch: String,
}

#[actix_web_codegen_const_routes::post(
    path = "PAGES.dash.site.add",
    wrap = "get_auth_middleware()"
)]
#[tracing::instrument(name = "Post Dashboard add site webpage", skip(ctx, id))]
pub async fn post_add_site(
    ctx: AppCtx,
    id: Identity,
    payload: web::Form<TemplateAddSite>,
) -> PageResult<impl Responder, Add> {
    let owner = id.identity().unwrap();
    let payload = payload.into_inner();
    let msg = AddSite {
        branch: payload.branch,
        repo_url: payload.repo_url,
        owner,
    };
    let page = ctx
        .add_site(msg)
        .await
        .map_err(|e| PageError::new(Add::new(&ctx.settings), e))?;

    Ok(HttpResponse::Found()
        .append_header((
            http::header::LOCATION,
            PAGES.dash.site.get_view(page.pub_id),
        ))
        .finish())
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(get_add_site);
    cfg.service(post_add_site);
}

#[cfg(test)]
mod tests {
    use actix_web::http::StatusCode;
    use actix_web::test;

    use crate::ctx::ArcCtx;
    use crate::pages::dash::sites::add::TemplateAddSite;
    use crate::tests;
    use crate::*;

    use super::PAGES;

    #[actix_rt::test]
    async fn postgres_dashboard_add_site_works() {
        let (_, ctx) = tests::get_ctx().await;
        dashboard_add_site_works(ctx.clone()).await;
    }

    async fn dashboard_add_site_works(ctx: ArcCtx) {
        const NAME: &str = "testdashaddsiteuser";
        const EMAIL: &str = "testdashaddsiteuser@foo.com";
        const PASSWORD: &str = "longpassword";

        let _ = ctx.delete_user(NAME, PASSWORD).await;
        let (_, signin_resp) = ctx.register_and_signin(NAME, EMAIL, PASSWORD).await;
        let cookies = get_cookie!(signin_resp);
        let app = get_app!(ctx.clone()).await;

        let resp = get_request!(&app, PAGES.dash.site.add, cookies.clone());
        assert_eq!(resp.status(), StatusCode::OK);
        let res = String::from_utf8(test::read_body(resp).await.to_vec()).unwrap();
        assert!(res.contains("Add Site"));

        let payload = TemplateAddSite {
            repo_url: tests::REPO_URL.into(),
            branch: tests::BRANCH.into(),
        };

        let add_site = test::call_service(
            &app,
            post_request!(&payload, PAGES.dash.site.add, FORM)
                .cookie(cookies.clone())
                .to_request(),
        )
        .await;
        assert_eq!(add_site.status(), StatusCode::FOUND);

        let mut site = ctx.db.list_all_sites(NAME).await.unwrap();
        let site = site.pop().unwrap();

        let mut event = ctx.db.list_all_site_events(&site.hostname).await.unwrap();
        let event = event.pop().unwrap();

        let headers = add_site.headers();
        let view_site = &PAGES.dash.site.get_view(site.pub_id.clone());
        assert_eq!(
            headers.get(actix_web::http::header::LOCATION).unwrap(),
            view_site
        );

        // view site
        let resp = get_request!(&app, view_site, cookies.clone());
        assert_eq!(resp.status(), StatusCode::OK);
        let res = String::from_utf8(test::read_body(resp).await.to_vec()).unwrap();
        assert!(res.contains(&site.site_secret));
        assert!(res.contains(&site.hostname));
        assert!(res.contains(&site.repo_url));
        assert!(res.contains(&site.branch));

        assert!(res.contains(&event.event_type.name));
        assert!(res.contains(&event.id.to_string()));

        let _ = ctx.delete_user(NAME, PASSWORD).await;
    }
}
