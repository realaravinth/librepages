/*
 * Copyright (C) 2021  Aravinth Manivannan <realaravinth@batsense.net>
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

#[derive(Debug, Clone, Deserialize)]
pub struct Page {
    pub secret: String,
    pub repo: String,
    pub path: String,
    pub branch: String,
}

impl Page {
    pub fn create_repo(&self) -> Repository {
        let repo = Repository::open(&self.path);

        if let Ok(repo) = repo {
            return repo;
        } else {
            info!("Cloning repository {} at {}", self.repo, self.path);
            Repository::clone(&self.repo, &self.path).unwrap()
        };
        //        let branch = repo.find_branch(&self.branch, BranchType::Local).unwrap();

        //repo.branches(BranchType::Local).unwrap().find(|b| b.unwrap().na
        let repo = Repository::open(&self.path).unwrap();
        {
            self._fetch_upstream(&repo, &self.branch);
            let branch = repo
                .find_branch(&format!("origin/{}", &self.branch), BranchType::Remote)
                .unwrap();

            let mut checkout_options = CheckoutBuilder::new();
            checkout_options.force();

            let tree = branch.get().peel(ObjectType::Tree).unwrap();

            repo.checkout_tree(&tree, Some(&mut checkout_options))
                .unwrap();
            //                repo.set_head(&format!("refs/heads/{}", &self.branch))
            //                    .unwrap();

            repo.set_head(branch.get().name().unwrap()).unwrap();
            //           }
        }
        repo
    }

    fn _fetch_upstream(&self, repo: &Repository, branch: &str) {
        let mut remote = repo.find_remote("origin").unwrap();
        remote.connect(Direction::Fetch).unwrap();
        info!("Updating repository {}", self.repo);
        remote.fetch(&[branch], None, None).unwrap();
        remote.disconnect().unwrap();
    }

    pub fn fetch_upstream(&self, branch: &str) {
        let repo = self.create_repo();
        self._fetch_upstream(&repo, branch);
    }
}
