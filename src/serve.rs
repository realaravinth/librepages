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
use crate::page::Page;
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

pub fn find_page<'a>(domain: &str, ctx: &'a AppCtx) -> Option<&'a Page> {
    log::info!("looking for {domain}");
    for page in ctx.settings.pages.iter() {
        log::debug!("configured domains: {}", page.domain);
        log::debug!("{}", page.domain.trim() == domain.trim());
        if page.domain.trim() == domain.trim() {
            log::debug!("found configured domains: {}", page.domain);
            return Some(page);
        }
    }
    None
}

#[my_codegen::get(path = "crate::V1_API_ROUTES.serve.catch_all")]
async fn index(req: HttpRequest, ctx: AppCtx) -> ServiceResult<impl Responder> {
    let c = req.connection_info();
    let mut host = c.host();
    if host.contains(':') {
        host = host.split(':').next().unwrap();
    }

    if host == ctx.settings.server.domain || host == "localhost" {
        return Ok(HttpResponse::Ok()
            .content_type(ContentType::html())
            .body("Welcome to Librepages!"));
    }

    if host.contains(&ctx.settings.server.domain) {
        let extractor = crate::preview::Preview::new(&ctx);
        if let Some(preview_branch) = extractor.extract(host) {
            unimplemented!(
                "map a local subdomain on settings.server.domain and use it to fetch page"
            );
            let res = match find_page(host, &ctx) {
                Some(page) => {
                    log::debug!("Page found");
                    let content = crate::git::read_preview_file(
                        &page.path,
                        preview_branch,
                        req.uri().path(),
                    )?;
                    let mime = if let Some(mime) = content.mime.first_raw() {
                        mime
                    } else {
                        "text/html; charset=utf-8"
                    };

                    Ok(HttpResponse::Ok()
                        //.content_type(ContentType::html())
                        .content_type(mime)
                        .body(content.content.bytes()))
                }
                None => Err(ServiceError::WebsiteNotFound),
            };
            return res;
        }
    }

    match find_page(host, &ctx) {
        Some(page) => {
            log::debug!("Page found");
            let content = crate::git::read_file(&page.path, req.uri().path())?;
            let mime = if let Some(mime) = content.mime.first_raw() {
                mime
            } else {
                "text/html; charset=utf-8"
            };

            Ok(HttpResponse::Ok()
                //.content_type(ContentType::html())
                .content_type(mime)
                .body(content.content.bytes()))
        }
        None => Err(ServiceError::WebsiteNotFound),
    }
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(index);
}
