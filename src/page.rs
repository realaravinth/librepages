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
use git2::{build::CheckoutBuilder, BranchType, Direction, ObjectType, Repository};
use log::info;
use serde::Deserialize;

use crate::errors::*;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Page {
    pub secret: String,
    pub repo: String,
    pub path: String,
    pub branch: String,
}

impl Page {
    fn create_repo(&self) -> ServiceResult<Repository> {
        let repo = Repository::open(&self.path);

        if let Ok(repo) = repo {
            return Ok(repo);
        } else {
            info!("Cloning repository {} at {}", self.repo, self.path);
            Repository::clone(&self.repo, &self.path)?;
        };

        let repo = Repository::open(&self.path)?;
        self._fetch_upstream(&repo, &self.branch)?;
        self.deploy_branch(&repo)?;
        Ok(repo)
    }

    pub fn deploy_branch(&self, repo: &Repository) -> ServiceResult<()> {
        let branch = repo.find_branch(&format!("origin/{}", &self.branch), BranchType::Remote)?;

        let mut checkout_options = CheckoutBuilder::new();
        checkout_options.force();

        let tree = branch.get().peel(ObjectType::Tree)?;

        repo.checkout_tree(&tree, Some(&mut checkout_options))?;
        repo.set_head(branch.get().name().unwrap())?;
        info!("Deploying branch {}", self.branch);
        Ok(())
    }

    fn _fetch_upstream(&self, repo: &Repository, branch: &str) -> ServiceResult<()> {
        let mut remote = repo.find_remote("origin").unwrap();
        remote.connect(Direction::Fetch)?;
        info!("Updating repository {}", self.repo);
        remote.fetch(&[branch], None, None)?;
        remote.disconnect()?;
        Ok(())
    }

    pub fn update(&self) -> ServiceResult<()> {
        let repo = self.create_repo()?;
        self._fetch_upstream(&repo, &self.branch)?;
        self.deploy_branch(&repo)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use git2::Branch;
    use git2::Repository;
    use mktemp::Temp;

    impl Page {
        fn get_tree<'a>(&self, repo: &'a Repository) -> Branch<'a> {
            repo.find_branch(&format!("origin/{}", &self.branch), BranchType::Remote)
                .unwrap()
        }
    }

    #[actix_rt::test]
    async fn pages_works() {
        let tmp_dir = Temp::new_dir().unwrap();
        assert!(tmp_dir.exists(), "tmp directory successully created");
        let mut page = Page {
            secret: String::default(),
            repo: "https://github.com/mcaptcha/website".to_owned(),
            path: tmp_dir.to_str().unwrap().to_string(),
            branch: "gh-pages".to_string(),
        };

        assert!(
            Repository::open(tmp_dir.as_path()).is_err(),
            "repository doesn't exist yet"
        );

        let repo = page.create_repo().unwrap();
        assert!(!repo.is_bare(), "repository isn't bare");
        page.create_repo().unwrap();
        assert!(
            Repository::open(tmp_dir.as_path()).is_ok(),
            "repository exists yet"
        );

        let gh_pages = page.get_tree(&repo);
        assert_eq!(
            gh_pages.name().unwrap().as_ref().unwrap(),
            &"origin/gh-pages"
        );
        page.branch = "master".to_string();
        page.update().unwrap();
        let master = page.get_tree(&repo);
        assert_eq!(master.name().unwrap().as_ref().unwrap(), &"origin/master");
    }
}
