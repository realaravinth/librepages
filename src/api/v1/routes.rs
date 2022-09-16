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
//! V1 API Routes
use actix_auth_middleware::GetLoginRoute;

use crate::deploy::routes::Deploy;
use crate::meta::routes::Meta;
use crate::serve::routes::Serve;

/// constant [Routes](Routes) instance
pub const ROUTES: Routes = Routes::new();

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
        let login = "/api/v1/signin";
        let logout = "/api/v1/logout";
        let register = "/api/v1/signup";
        Auth {
            logout,
            login,
            register,
        }
    }
}

/// Account management routes
pub struct Account {
    /// delete account route
    pub delete: &'static str,
    /// route to check if an email exists
    pub email_exists: &'static str,
    /// route to update a user's email
    pub update_email: &'static str,
    ///    route to update password
    pub update_password: &'static str,
    ///    route to check if a username is already registered
    pub username_exists: &'static str,
    ///    route to change username
    pub update_username: &'static str,
}

impl Account {
    /// create a new instance of [Account][Account] routes
    pub const fn new() -> Account {
        let delete = "/api/v1/account/delete";
        let email_exists = "/api/v1/account/email/exists";
        let username_exists = "/api/v1/account/username/exists";
        let update_username = "/api/v1/account/username/update";
        let update_email = "/api/v1/account/email/update";
        let update_password = "/api/v1/account/password/update";
        Account {
            delete,
            email_exists,
            update_email,
            update_password,
            username_exists,
            update_username,
        }
    }
}

/// Top-level routes data structure for V1 AP1
pub struct Routes {
    /// Authentication routes
    pub auth: Auth,
    /// Account routes
    pub account: Account,
    /// Meta routes
    pub meta: Meta,
    pub deploy: Deploy,
    pub serve: Serve,
}

impl Routes {
    /// create new instance of Routes
    const fn new() -> Routes {
        Routes {
            auth: Auth::new(),
            account: Account::new(),
            meta: Meta::new(),
            deploy: Deploy::new(),
            serve: Serve::new(),
        }
    }
}

impl GetLoginRoute for Routes {
    fn get_login_route(&self, src: Option<&str>) -> String {
        if let Some(redirect_to) = src {
            format!(
                "{}?redirect_to={}",
                self.auth.login,
                urlencoding::encode(redirect_to)
            )
        } else {
            self.auth.register.to_string()
        }
    }
}
