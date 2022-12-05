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
use actix_auth_middleware::{Authentication, GetLoginRoute};
use serde::*;
use uuid::Uuid;

/// constant [Pages](Pages) instance
pub const PAGES: Pages = Pages::new();

#[derive(Serialize)]
/// Top-level routes data structure for V1 AP1
pub struct Pages {
    /// Authentication routes
    pub auth: Auth,
    /// home page
    pub home: &'static str,
    pub dash: Dash,
}

impl Pages {
    /// create new instance of Routes
    const fn new() -> Pages {
        let auth = Auth::new();
        let dash = Dash::new();
        let home = auth.login;
        Pages { auth, home, dash }
    }
}

#[derive(Serialize)]
/// Authentication routes
pub struct Auth {
    /// logout route
    pub logout: &'static str,
    /// login route
    pub login: &'static str,
    /// registration route
    pub register: &'static str,
}

impl Auth {
    /// create new instance of Authentication route
    pub const fn new() -> Auth {
        let login = "/login";
        let logout = "/logout";
        let register = "/join";
        Auth {
            logout,
            login,
            register,
        }
    }
}

#[derive(Serialize)]
/// Dashboard routes
pub struct Dash {
    /// home route
    pub home: &'static str,
    pub site: DashSite,
}

impl Dash {
    /// create new instance of Dash route
    pub const fn new() -> Dash {
        let home = "/dash";
        let site = DashSite::new();
        Dash { home, site }
    }
}

#[derive(Serialize)]
/// Dashboard Site routes
pub struct DashSite {
    /// add site route
    pub add: &'static str,
    /// view site route
    pub view: &'static str,
}

impl DashSite {
    /// create new instance of DashSite route
    pub const fn new() -> DashSite {
        let add = "/dash/site/add";
        let view = "/dash/site/view/{deployment_pub_id}";
        DashSite { add, view }
    }

    pub fn get_view(&self, deployment_pub_id: Uuid) -> String {
        self.view.replace(
            "{deployment_pub_id}",
            deployment_pub_id.to_string().as_ref(),
        )
    }
}

pub fn get_auth_middleware() -> Authentication<Pages> {
    Authentication::with_identity(PAGES)
}

impl GetLoginRoute for Pages {
    fn get_login_route(&self, src: Option<&str>) -> String {
        if let Some(redirect_to) = src {
            format!(
                "{}?redirect_to={}",
                self.auth.login,
                urlencoding::encode(redirect_to)
            )
        } else {
            self.auth.login.to_string()
        }
    }
}
