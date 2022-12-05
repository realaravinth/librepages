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

use actix_web::{
    body::{BoxBody, EitherBody},
    dev::ServiceResponse,
    error::ResponseError,
    http::StatusCode,
};
use mktemp::Temp;
use serde::Serialize;

use crate::ctx::api::v1::auth::{Login, Register};
use crate::ctx::api::v1::pages::AddSite;
use crate::ctx::Ctx;
use crate::errors::*;
use crate::page::Page;
use crate::settings::Settings;
use crate::*;

pub const REPO_URL: &str = "http://localhost:8080/mCaptcha/website/";
pub const BRANCH: &str = "gh-pages";

pub async fn get_ctx() -> (Temp, Arc<Ctx>) {
    // mktemp::Temp is returned because the temp directory created
    // is removed once the variable goes out of scope
    let mut settings = Settings::new().unwrap();

    let tmp_dir = Temp::new_dir().unwrap();
    println!("[log] Test temp directory: {}", tmp_dir.to_str().unwrap());
    let page_base_path = tmp_dir.as_path().join("base_path");
    settings.page.base_path = page_base_path.to_str().unwrap().into();
    settings.init();
    println!("[log] Initialzing settings again with test config");
    settings.init();

    (tmp_dir, Ctx::new(settings).await)
}

#[allow(dead_code, clippy::upper_case_acronyms)]
pub struct FORM;

#[macro_export]
macro_rules! post_request {
    ($uri:expr) => {
        actix_web::test::TestRequest::post().uri($uri)
    };

    ($serializable:expr, $uri:expr) => {
        actix_web::test::TestRequest::post()
            .uri($uri)
            .insert_header((actix_web::http::header::CONTENT_TYPE, "application/json"))
            .set_payload(serde_json::to_string($serializable).unwrap())
    };

    ($serializable:expr, $uri:expr, FORM) => {
        actix_web::test::TestRequest::post()
            .uri($uri)
            .set_form($serializable)
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

    ($app:expr, $route:expr, $cookies:expr, $serializable:expr, FORM) => {
        test::call_service(
            &$app,
            test::TestRequest::delete()
                .uri($route)
                .set_form($serializable)
                .cookie($cookies)
                .to_request(),
        )
        .await
    };
}

#[macro_export]
macro_rules! get_app {
    ($ctx:expr) => {
        actix_web::test::init_service(
            actix_web::App::new()
                .app_data($crate::get_json_err())
                .wrap($crate::get_identity_service(&$ctx.settings))
                .wrap(actix_web::middleware::NormalizePath::new(
                    actix_web::middleware::TrailingSlash::Trim,
                ))
                .configure($crate::services)
                .app_data($crate::WebData::new($ctx.clone())),
        )
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

#[macro_export]
macro_rules! get_cookie {
    ($resp:expr) => {
        $resp.response().cookies().next().unwrap().to_owned()
    };
}

impl Ctx {
    /// register and signin utility
    pub async fn register_and_signin(
        &self,
        name: &str,
        email: &str,
        password: &str,
    ) -> (Login, ServiceResponse<EitherBody<BoxBody>>) {
        self.register_test(name, email, password).await;
        self.signin_test(name, password).await
    }

    pub fn to_arc(&self) -> Arc<Self> {
        Arc::new(self.clone())
    }

    /// register utility
    pub async fn register_test(&self, name: &str, email: &str, password: &str) {
        let app = get_app!(self.to_arc()).await;

        // 1. Register
        let msg = Register {
            username: name.into(),
            password: password.into(),
            confirm_password: password.into(),
            email: email.into(),
        };
        println!("{:?}", msg);
        let resp = actix_web::test::call_service(
            &app,
            post_request!(&msg, crate::V1_API_ROUTES.auth.register).to_request(),
        )
        .await;
        if resp.status() != StatusCode::OK {
            let resp_err: ErrorToResponse = actix_web::test::read_body_json(resp).await;
            panic!("{}", resp_err.error);
        }
    }

    /// signin util
    pub async fn signin_test(
        &self,

        name: &str,
        password: &str,
    ) -> (Login, ServiceResponse<EitherBody<BoxBody>>) {
        let app = get_app!(self.to_arc()).await;

        // 2. signin
        let creds = Login {
            login: name.into(),
            password: password.into(),
        };
        let signin_resp = actix_web::test::call_service(
            &app,
            post_request!(&creds, V1_API_ROUTES.auth.login).to_request(),
        )
        .await;
        assert_eq!(signin_resp.status(), StatusCode::OK);
        (creds, signin_resp)
    }

    /// pub duplicate test
    pub async fn bad_post_req_test<T: Serialize>(
        &self,

        name: &str,
        password: &str,
        url: &str,
        payload: &T,
        err: ServiceError,
    ) {
        let (_, signin_resp) = self.signin_test(name, password).await;
        let cookies = get_cookie!(signin_resp);
        let app = get_app!(self.to_arc()).await;

        let resp = actix_web::test::call_service(
            &app,
            post_request!(&payload, url)
                .cookie(cookies.clone())
                .to_request(),
        )
        .await;
        assert_eq!(resp.status(), err.status_code());
        let resp_err: ErrorToResponse = actix_web::test::read_body_json(resp).await;
        //println!("{}", txt.error);
        assert_eq!(resp_err.error, format!("{}", err));
    }

    /// bad post req test without payload
    pub async fn bad_post_req_test_witout_payload(
        &self,
        name: &str,
        password: &str,
        url: &str,
        err: ServiceError,
    ) {
        let (_, signin_resp) = self.signin_test(name, password).await;
        let app = get_app!(self.to_arc()).await;
        let cookies = get_cookie!(signin_resp);

        let resp = actix_web::test::call_service(
            &app,
            post_request!(url).cookie(cookies.clone()).to_request(),
        )
        .await;
        assert_eq!(resp.status(), err.status_code());
        let resp_err: ErrorToResponse = actix_web::test::read_body_json(resp).await;
        //println!("{}", resp_err.error);
        assert_eq!(resp_err.error, format!("{}", err));
    }

    pub async fn add_test_site(&self, owner: String) -> Page {
        let msg = AddSite {
            repo_url: REPO_URL.into(),
            branch: BRANCH.into(),
            owner,
        };
        self.add_site(msg).await.unwrap()
    }
}
