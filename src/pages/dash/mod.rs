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

pub use super::{context, Footer, TemplateFile, PAGES, PAYLOAD_KEY, TEMPLATES};

mod home;

pub fn register_templates(t: &mut tera::Tera) {
    home::DASH_HOME.register(t).expect(home::DASH_HOME.name);
}

pub fn services(cfg: &mut web::ServiceConfig) {
    home::services(cfg);
}
