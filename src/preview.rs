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
use crate::AppCtx;

pub struct Preview<'a> {
    pub base: &'a str,
    pub delimiter: &'static str,
    pub prefix: &'static str,
}

impl<'a> Preview<'a> {
    pub fn new(ctx: &'a AppCtx) -> Self {
        Self {
            base: &ctx.settings.page.base_domain,
            delimiter: ".",
            prefix: "deploy-preview-",
        }
    }
    pub fn get_name(&self, preview_number: usize) -> String {
        format!(
            "{}{preview_number}{}{}",
            self.prefix, self.delimiter, self.base
        )
    }

    pub fn extract(&self, hostname: &'a str) -> Option<&'a str> {
        if !hostname.contains(self.delimiter)
            || !hostname.contains(self.prefix)
            || !hostname.contains(self.base)
        {
            return None;
        }

        let d = format!("{}{}", self.delimiter, self.base);

        if hostname.split(&d).count() == 2 {
            return hostname.split(&d).next();
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_extract_generate_works() {
        const BASE_DOMAIN: &str = "librepages.site";
        const PREVIEW_DELIMITER: &str = ".";
        const PREVIEW_PREFIX: &str = "deploy-preview-";
        const PREVIEW_NUMBER: usize = 1666;

        let preview_hostname =
            format!("{PREVIEW_PREFIX}{PREVIEW_NUMBER}{PREVIEW_DELIMITER}{BASE_DOMAIN}");

        let bad_hostname = BASE_DOMAIN.to_string();

        let extractor = Preview {
            base: BASE_DOMAIN,
            prefix: PREVIEW_PREFIX,
            delimiter: PREVIEW_DELIMITER,
        };

        assert_eq!(extractor.get_name(PREVIEW_NUMBER), preview_hostname);

        assert_eq!(extractor.extract(&bad_hostname), None);

        assert_eq!(
            extractor.extract(&format!(
                "{PREVIEW_PREFIX}{PREVIEW_NUMBER}{PREVIEW_DELIMITER}no_base_domain"
            )),
            None
        );

        assert_eq!(
            extractor.extract(&format!(
                "{PREVIEW_PREFIX}{PREVIEW_NUMBER}no-delimiter{BASE_DOMAIN}"
            )),
            None
        );

        assert_eq!(
            extractor.extract(&format!(
                "{PREVIEW_PREFIX}{PREVIEW_NUMBER}no-delimiter{BASE_DOMAIN}"
            )),
            None
        );

        assert_eq!(
            extractor.extract(&format!(
                "noprefix{PREVIEW_NUMBER}{PREVIEW_DELIMITER}{BASE_DOMAIN}"
            )),
            None
        );

        assert_eq!(
            extractor.extract(&preview_hostname),
            Some(format!("{PREVIEW_PREFIX}{PREVIEW_NUMBER}").as_str())
        );
    }
}
