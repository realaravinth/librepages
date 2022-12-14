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
use std::thread;

use crate::db::*;
use crate::settings::Settings;
use argon2_creds::{Config as ArgonConfig, ConfigBuilder as ArgonConfigBuilder, PasswordPolicy};
use tracing::info;

pub mod api;

use crate::conductor::Conductor;

pub type ArcCtx = Arc<Ctx>;

#[derive(Clone)]
pub struct Ctx {
    pub settings: Settings,
    pub db: Database,
    pub conductor: Conductor,
    /// credential-procession policy
    pub creds: ArgonConfig,
}

impl Ctx {
    /// Get credential-processing policy
    pub fn get_creds() -> ArgonConfig {
        ArgonConfigBuilder::default()
            .username_case_mapped(true)
            .profanity(true)
            .blacklist(true)
            .password_policy(PasswordPolicy::default())
            .build()
            .unwrap()
    }

    pub async fn new(settings: Settings) -> Arc<Self> {
        let creds = Self::get_creds();
        let c = creds.clone();
        let conductor = Conductor::new(settings.clone());

        #[allow(unused_variables)]
        let init = thread::spawn(move || {
            info!("Initializing credential manager");
            c.init();
            info!("Initialized credential manager");
        });
        let db = get_db(&settings).await;

        #[cfg(not(debug_assertions))]
        init.join();

        Arc::new(Self {
            settings,
            db,
            creds,
            conductor,
        })
    }
}
