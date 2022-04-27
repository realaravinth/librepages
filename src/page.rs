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
use git2::{
    build::CheckoutBuilder, Branch, BranchType, Direction, ObjectType, Oid, Remote, Repository,
};
#[cfg(not(test))]
use log::info;

#[cfg(test)]
use std::println as info;

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
    pub fn open_repo(&self) -> ServiceResult<Repository> {
        Ok(Repository::open(&self.path)?)
    }

    fn create_repo(&self) -> ServiceResult<Repository> {
        let repo = self.open_repo();

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

    fn find_branch<'a>(&self, repo: &'a Repository) -> ServiceResult<Branch<'a>> {
        let branch = repo.find_branch(&self.branch, BranchType::Local)?;
        Ok(branch)
    }

    pub fn deploy_branch(&self, repo: &Repository) -> ServiceResult<()> {
        let branch = self.find_branch(repo)?;

        let mut checkout_options = CheckoutBuilder::new();
        checkout_options.force();

        let tree = branch.get().peel(ObjectType::Tree)?;

        repo.checkout_tree(&tree, Some(&mut checkout_options))?;
        repo.set_head(&format!("refs/heads/{}", self.branch))?;
        info!("Deploying branch {}", self.branch);
        Ok(())
    }

    fn _fetch_upstream(&self, repo: &Repository, branch: &str) -> ServiceResult<()> {
        let mut remote = Self::get_deploy_remote(repo)?;
        remote.connect(Direction::Fetch)?;
        info!("Updating repository {}", self.repo);
        let remote_branch_name = format!("origin/{branch}");
        remote.fetch(&[&remote_branch_name], None, None)?;
        remote.disconnect()?;
        let branch = repo.find_branch(&remote_branch_name, BranchType::Remote)?;
        let commit = branch.get().peel_to_commit()?;
        if self.find_branch(repo).is_err() {
            repo.branch(&self.branch, &commit, true)?;
        }
        Ok(())
    }

    pub fn update(&self) -> ServiceResult<()> {
        let repo = self.create_repo()?;
        self._fetch_upstream(&repo, &self.branch)?;
        self.deploy_branch(&repo)?;
        Ok(())
    }

    pub fn get_deploy_branch(&self, repo: &Repository) -> ServiceResult<String> {
        let branch = self.find_branch(repo)?;
        if branch.is_head() {
            Ok(self.branch.clone())
        } else {
            Err(ServiceError::BranchNotFound(self.branch.clone()))
        }
    }

    pub fn get_deploy_commit(repo: &Repository) -> ServiceResult<Oid> {
        let head = repo.head()?;
        let commit = head.peel_to_commit()?;
        Ok(commit.id())
    }

    pub fn get_deploy_remote(repo: &Repository) -> ServiceResult<Remote> {
        Ok(repo.find_remote("origin")?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use git2::Repository;
    use mktemp::Temp;

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

        let gh_pages = page.get_deploy_branch(&repo).unwrap();
        assert_eq!(gh_pages, "gh-pages");
        page.branch = "master".to_string();
        page.update().unwrap();
        let master = page.get_deploy_branch(&repo).unwrap();
        assert_eq!(master, "master");

        assert_eq!(
            Page::get_deploy_remote(&repo).unwrap().url().unwrap(),
            page.repo
        );
    }
}
