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
use std::sync::Arc;

use crate::ctx::Ctx;
use crate::settings::Settings;

pub async fn get_data() -> Arc<Ctx> {
    let settings = Settings::new().unwrap();
    Ctx::new(settings)
}

#[allow(dead_code, clippy::upper_case_acronyms)]
pub struct FORM;

#[macro_export]
macro_rules! post_request {
    ($uri:expr) => {
        test::TestRequest::post().uri($uri)
    };

    ($serializable:expr, $uri:expr) => {
        test::TestRequest::post()
            .uri($uri)
            .insert_header((actix_web::http::header::CONTENT_TYPE, "application/json"))
            .set_payload(serde_json::to_string($serializable).unwrap())
    };

    ($serializable:expr, $uri:expr, FORM) => {
        test::TestRequest::post().uri($uri).set_form($serializable)
    };
}

#[macro_export]
macro_rules! get_request {
    ($app:expr,$route:expr ) => {
        test::call_service(&$app, test::TestRequest::get().uri($route).to_request()).await
    };

    ($app:expr, $route:expr, $cookies:expr) => {
        test::call_service(
            &$app,
            test::TestRequest::get()
                .uri($route)
                .cookie($cookies)
                .to_request(),
        )
        .await
    };
}

#[macro_export]
macro_rules! delete_request {
    ($app:expr,$route:expr ) => {
        test::call_service(&$app, test::TestRequest::delete().uri($route).to_request()).await
    };

    ($app:expr, $route:expr, $cookies:expr) => {
        test::call_service(
            &$app,
            test::TestRequest::delete()
                .uri($route)
                .cookie($cookies)
                .to_request(),
        )
        .await
    };
}

#[macro_export]
macro_rules! get_app {
    ("APP") => {
        actix_web::App::new()
            .app_data(crate::get_json_err())
            .wrap(actix_web::middleware::NormalizePath::new(
                actix_web::middleware::TrailingSlash::Trim,
            ))
            .configure(crate::routes::services)
    };

    //    ($settings:ident) => {
    //        test::init_service(get_app!("APP", $settings))
    //    };
    ($ctx:expr) => {
        test::init_service(get_app!("APP").app_data(crate::WebData::new($ctx.clone())))
    };
}
