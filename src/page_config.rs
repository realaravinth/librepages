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

use libconfig::Config;
use serde::{Deserialize, Serialize};

use crate::git::{ContentType, GitFileMode};

#[derive(Deserialize, Debug, Serialize, PartialEq, Eq)]
struct Policy<'a> {
    rel_path: &'a str,
    format: SupportedFormat,
}

impl<'a> Policy<'a> {
    const fn new(rel_path: &'a str, format: SupportedFormat) -> Self {
        Self { rel_path, format }
    }
}

#[derive(Deserialize, Debug, Serialize, PartialEq, Eq)]
enum SupportedFormat {
    Json,
    Yaml,
    Toml,
}

pub fn load<P: AsRef<Path>>(repo_path: &P, branch: &str) -> Option<Config> {
    const POLICIES: [Policy; 2] = [
        Policy::new("librepages.toml", SupportedFormat::Toml),
        Policy::new("librepages.json", SupportedFormat::Json),
    ];

    if let Some(policy) = discover(repo_path, branch, &POLICIES) {
        //            let path = p.repo.as_ref().join(policy.rel_path);
        //let contents = fs::read_to_string(path).await.unwrap();

        let file =
            crate::git::read_preview_file(&repo_path.as_ref().into(), branch, policy.rel_path)
                .unwrap();
        if let ContentType::Text(contents) = file.content {
            let res = match policy.format {
                SupportedFormat::Json => load_json(&contents),
                SupportedFormat::Yaml => load_yaml(&contents),
                SupportedFormat::Toml => load_toml(&contents),
            };

            return Some(res);
        };
    }

    None
}
fn discover<'a, P: AsRef<Path>>(
    repo_path: &P,
    branch: &str,
    policies: &'a [Policy<'a>],
) -> Option<&'a Policy<'a>> {
    let repo = git2::Repository::open(repo_path).unwrap();

    let branch = repo.find_branch(branch, git2::BranchType::Local).unwrap();
    //    let tree = head.peel_to_tree().unwrap();
    let branch = branch.into_reference();
    let tree = branch.peel_to_tree().unwrap();

    for p in policies.iter() {
        let file_exists = tree.iter().any(|x| {
            if let Some(name) = x.name() {
                if policies.iter().any(|p| p.rel_path == name) {
                    let mode: GitFileMode = x.into();
                    matches!(mode, GitFileMode::Executable | GitFileMode::Regular)
                } else {
                    false
                }
            } else {
                false
            }
        });

        if file_exists {
            return Some(p);
        }
    }
    None
}

fn load_toml(c: &str) -> Config {
    toml::from_str(c).unwrap()
}

fn load_yaml(c: &str) -> Config {
    serde_yaml::from_str(c).unwrap()
}

fn load_json(c: &str) -> Config {
    serde_json::from_str(c).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::tests::write_file_util;
    use mktemp::Temp;

    use libconfig::*;

    #[actix_rt::test]
    async fn page_config_test() {
        let tmp_dir = Temp::new_dir().unwrap();
        let repo_path = tmp_dir.join("page_config_test");

        let content = std::fs::read_to_string(
            &Path::new("./tests/cases/contains-everything/toml/librepages.toml")
                .canonicalize()
                .unwrap(),
        )
        .unwrap();

        write_file_util(
            repo_path.to_str().unwrap(),
            "librepages.toml",
            Some(&content),
        );

        let config = load(&repo_path, "master").unwrap();
        assert!(config.forms.as_ref().unwrap().enable);
        assert!(config.image_compression.as_ref().unwrap().enable);
        assert_eq!(config.source.production_branch, "librepages");
        assert_eq!(config.source.staging.as_ref().unwrap(), "beta");

        assert_eq!(
            config.redirects.as_ref().unwrap(),
            &vec![
                Redirects {
                    from: "/from1".into(),
                    to: "/to1".into()
                },
                Redirects {
                    from: "/from2".into(),
                    to: "/to2".into()
                },
            ]
        );

        assert_eq!(
            config.domains.as_ref().unwrap(),
            &vec!["example.org".to_string(), "example.com".to_string(),]
        );
    }
}
