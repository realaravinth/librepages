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
use actix_web::{http::header::ContentType, web, HttpRequest, HttpResponse, Responder};

use crate::errors::*;
use crate::AppCtx;

pub mod routes {
    pub struct Serve {
        pub catch_all: &'static str,
    }

    impl Serve {
        pub const fn new() -> Self {
            Self {
                catch_all: "/{path:.*}",
            }
        }
    }
}

#[actix_web_codegen_const_routes::get(path = "crate::V1_API_ROUTES.serve.catch_all")]
async fn index(req: HttpRequest, ctx: AppCtx) -> ServiceResult<impl Responder> {
    let c = req.connection_info();
    let mut host = c.host();
    if host.contains(':') {
        host = host.split(':').next().unwrap();
    }

    // serve meta page
    if host == ctx.settings.server.domain || host == "localhost" {
        return Ok(HttpResponse::Ok()
            .content_type(ContentType::html())
            .body("Welcome to Librepages!"));
    }

    // serve default hostname content
    if host.contains(&ctx.settings.page.base_domain) {
        let extractor = crate::preview::Preview::new(&ctx);
        if let Some(preview_branch) = extractor.extract(host) {
            let res = if ctx.db.hostname_exists(&host).await? {
                let path = crate::utils::get_website_path(&ctx.settings, &host);
                let content =
                    crate::git::read_preview_file(&path, preview_branch, req.uri().path())?;
                let mime = if let Some(mime) = content.mime.first_raw() {
                    mime
                } else {
                    "text/html; charset=utf-8"
                };

                Ok(HttpResponse::Ok()
                    .content_type(mime)
                    .body(content.content.bytes()))
            } else {
                Err(ServiceError::WebsiteNotFound)
            };
            return res;
        }
    }

    // TODO: custom domains.
    if ctx.db.hostname_exists(host).await? {
        let path = crate::utils::get_website_path(&ctx.settings, &host);
        let content = crate::git::read_file(&path, req.uri().path())?;
        let mime = if let Some(mime) = content.mime.first_raw() {
            mime
        } else {
            "text/html; charset=utf-8"
        };

        Ok(HttpResponse::Ok()
            .content_type(mime)
            .body(content.content.bytes()))
    } else {
        Err(ServiceError::WebsiteNotFound)
    }
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(index);
}
