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
use actix_web::*;

use super::get_auth_middleware;
pub use super::home::TemplateSite;
pub use super::{context, Footer, TemplateFile, PAGES, PAYLOAD_KEY, TEMPLATES};

pub mod add;
pub mod delete;
pub mod view;

pub fn register_templates(t: &mut tera::Tera) {
    add::DASH_SITE_ADD
        .register(t)
        .expect(add::DASH_SITE_ADD.name);
    view::DASH_SITE_VIEW
        .register(t)
        .expect(view::DASH_SITE_VIEW.name);
    delete::DASH_SITE_DELETE
        .register(t)
        .expect(delete::DASH_SITE_DELETE.name);
}

pub fn services(cfg: &mut web::ServiceConfig) {
    add::services(cfg);
    view::services(cfg);
    delete::services(cfg);
}
