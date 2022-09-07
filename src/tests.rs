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
use std::path::Path;
use std::sync::Arc;

use mktemp::Temp;

use crate::ctx::Ctx;
use crate::page::Page;
use crate::settings::Settings;

pub async fn get_data() -> (Temp, Arc<Ctx>) {
    // mktemp::Temp is returned because the temp directory created
    // is removed once the variable goes out of scope
    let mut settings = Settings::new().unwrap();

    let tmp_dir = Temp::new_dir().unwrap();
    println!("[log] Test temp directory: {}", tmp_dir.to_str().unwrap());
    let mut pages = Vec::with_capacity(settings.pages.len());
    for page in settings.pages.iter() {
        let name = Path::new(&page.path).file_name().unwrap().to_str().unwrap();
        let path = tmp_dir.as_path().join(name);
        let page = Page {
            path: path.to_str().unwrap().to_string(),
            secret: page.secret.clone(),
            branch: page.branch.clone(),
            repo: page.repo.clone(),
            domain: "mcaptcha.org".into(),
        };

        pages.push(Arc::new(page));
    }

    settings.pages = pages;
    println!("[log] Initialzing settings again with test config");
    settings.init();

    (tmp_dir, Ctx::new(settings))
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
            .app_data($crate::get_json_err())
            .wrap(actix_web::middleware::NormalizePath::new(
                actix_web::middleware::TrailingSlash::Trim,
            ))
            .configure($crate::routes::services)
    };

    //    ($settings:ident) => {
    //        test::init_service(get_app!("APP", $settings))
    //    };
    ($ctx:expr) => {
        test::init_service(get_app!("APP").app_data($crate::WebData::new($ctx.clone())))
    };
}

/// Utility function to check for status of a test response, attempt response payload serialization
/// and print payload if response status doesn't match expected status
#[macro_export]
macro_rules! check_status {
    ($resp:expr, $expected:expr) => {
        let status = $resp.status();
        if status != $expected {
            eprintln!(
                "[error] Expected status code: {} received: {status}",
                $expected
            );
            let response: serde_json::Value = actix_web::test::read_body_json($resp).await;
            eprintln!("[error] Body:\n{:#?}", response);
            assert_eq!(status, $expected);
            panic!()
        }
        {
            assert_eq!(status, $expected);
        }
    };
}
