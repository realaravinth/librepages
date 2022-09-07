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

use git2::{build::CheckoutBuilder, BranchType, Direction, Oid, Remote, Repository};
#[cfg(not(test))]
use log::info;

#[cfg(test)]
use std::println as info;

use serde::Deserialize;

use crate::errors::*;

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
pub struct Page {
    pub secret: String,
    pub repo: String,
    pub path: String,
    pub branch: String,
    pub domain: String,
}

impl Page {
    pub fn open_repo(&self) -> ServiceResult<Repository> {
        Ok(Repository::open(&self.path)?)
    }

    fn create_repo(&self) -> ServiceResult<Repository> {
        let repo = self.open_repo();

        let repo = if let Ok(repo) = repo {
            repo
        } else {
            info!("Cloning repository {} at {}", self.repo, self.path);
            Repository::clone(&self.repo, &self.path)?;
            Repository::open(&self.path)?
        };

        self._fetch_remote_branch(&repo, &self.branch)?;
        self.deploy_branch(&repo).unwrap();

        Ok(repo)
    }

    pub fn deploy_branch(&self, repo: &Repository) -> ServiceResult<()> {
        let mut checkout_options = CheckoutBuilder::default();
        checkout_options
            .allow_conflicts(true)
            .conflict_style_merge(true)
            .force();

        let refname = format!("refs/heads/{}", self.branch);

        repo.set_head(&refname).unwrap();
        repo.checkout_head(Some(&mut checkout_options)).unwrap();

        info!("Deploying branch {}", self.branch);
        Ok(())
    }

    fn fetch<'a>(&self, repo: &'a git2::Repository) -> ServiceResult<git2::AnnotatedCommit<'a>> {
        let mut remote = repo.find_remote("origin")?;
        log::info!("Fetching {} for repo", remote.name().unwrap());
        remote.fetch(&[&self.branch], None, None)?;
        let fetch_head = repo.find_reference("FETCH_HEAD")?;
        Ok(repo.reference_to_annotated_commit(&fetch_head)?)
    }

    fn merge<'a>(
        &self,
        repo: &'a Repository,
        fetch_commit: git2::AnnotatedCommit<'a>,
    ) -> ServiceResult<()> {
        // 1. do a merge analysis
        let analysis = repo.merge_analysis(&[&fetch_commit])?;

        // 2. Do the appropriate merge
        if analysis.0.is_fast_forward() {
            //log::debug!("Doing a fast forward");
            log::debug!("Doing a fast forward");
            // do a fast forward
            let refname = format!("refs/heads/{}", &self.branch);
            match repo.find_reference(&refname) {
                Ok(mut r) => {
                    log::debug!("fast forwarding");
                    Self::fast_forward(repo, &mut r, &fetch_commit).unwrap();
                }
                Err(_) => {
                    // The branch doesn't exist so just set the reference to the
                    // commit directly. Usually this is because you are pulling
                    // into an empty repository.
                    log::error!("Error in find ref");
                    repo.reference(
                        &refname,
                        fetch_commit.id(),
                        true,
                        &format!("Setting {} to {}", &self.branch, fetch_commit.id()),
                    )
                    .unwrap();
                    repo.set_head(&refname).unwrap();
                    repo.checkout_head(Some(
                        git2::build::CheckoutBuilder::default()
                            .allow_conflicts(true)
                            .conflict_style_merge(true)
                            .force(),
                    ))
                    .unwrap();
                }
            };
        } else if analysis.0.is_normal() {
            // do a normal merge
            // expects repo.head to point to the branch when is going to receive merges
            let head_commit = repo
                .reference_to_annotated_commit(&repo.head().unwrap())
                .unwrap();
            Self::normal_merge(repo, &head_commit, &fetch_commit).unwrap();
        } else {
            log::info!("Nothing to do...");
        }
        Ok(())
    }

    fn _fetch_remote_branch(&self, repo: &Repository, branch: &str) -> ServiceResult<()> {
        let mut remote = Self::get_deploy_remote(repo)?;
        remote.connect(Direction::Fetch)?;
        info!("Updating repository {}", self.repo);
        let remote_branch_name = format!("origin/{branch}");
        remote.fetch(&[&remote_branch_name], None, None)?;
        remote.disconnect()?;
        let branch = repo.find_branch(&remote_branch_name, BranchType::Remote)?;
        let commit = branch.get().peel_to_commit()?;
        if repo.find_branch(&self.branch, BranchType::Local).is_err() {
            repo.branch(&self.branch, &commit, true)?;
        }
        Ok(())
    }

    fn normal_merge(
        repo: &Repository,
        local: &git2::AnnotatedCommit,
        remote: &git2::AnnotatedCommit,
    ) -> Result<(), git2::Error> {
        let local_tree = repo.find_commit(local.id())?.tree().unwrap();
        let remote_tree = repo.find_commit(remote.id())?.tree().unwrap();
        println!("{} {}", local.id(), remote.id());
        let ancestor = repo
            .find_commit(repo.merge_base(local.id(), remote.id()).unwrap())
            .unwrap()
            .tree()
            .unwrap();
        let mut idx = repo
            .merge_trees(&ancestor, &local_tree, &remote_tree, None)
            .unwrap();

        if idx.has_conflicts() {
            log::debug!("Merge conflicts detected...");
            repo.checkout_index(Some(&mut idx), None)?;
            return Ok(());
        }
        let result_tree = repo.find_tree(idx.write_tree_to(repo)?)?;
        // now create the merge commit
        let msg = format!("Merge: {} into {}", remote.id(), local.id());
        let sig = repo.signature()?;
        let local_commit = repo.find_commit(local.id())?;
        let remote_commit = repo.find_commit(remote.id())?;
        // Do our merge commit and set current branch head to that commit.
        let _merge_commit = repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            &msg,
            &result_tree,
            &[&local_commit, &remote_commit],
        )?;
        // Set working tree to match head.
        repo.checkout_head(None)?;
        Ok(())
    }

    fn fast_forward(
        repo: &Repository,
        lb: &mut git2::Reference,
        rc: &git2::AnnotatedCommit,
    ) -> ServiceResult<()> {
        let name = match lb.name() {
            Some(s) => s.to_string(),
            None => String::from_utf8_lossy(lb.name_bytes()).to_string(),
        };
        let msg = format!("Fast-Forward: Setting {} to id: {}", name, rc.id());
        log::debug!("{}", msg);
        lb.set_target(rc.id(), &msg)?;
        repo.set_head(&name)?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        Ok(())
    }

    pub fn update(&self) -> ServiceResult<()> {
        let repo = self.create_repo()?;
        let fetch_commit = self.fetch(&repo)?;
        self.merge(&repo, fetch_commit)?;
        Ok(())
    }

    pub fn get_deploy_branch(&self, repo: &Repository) -> ServiceResult<String> {
        let branch = repo.find_branch(&self.branch, BranchType::Local)?;
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
            domain: "mcaptcha.org".into(),
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
