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
use std::path::PathBuf;

use git2::*;
use mime_guess::MimeGuess;
use num_enum::FromPrimitive;
use serde::{Deserialize, Serialize};

use crate::errors::*;

/// A FileMode represents the kind of tree entries used by git. It
/// resembles regular file systems modes, although FileModes are
/// considerably simpler (there are not so many), and there are some,
/// like Submodule that has no file system equivalent.
// Adapted from https://github.com/go-git/go-git/blob/master/plumbing/filemode/filemode.go(Apache-2.0 License)
#[derive(Debug, PartialEq, Eq, Clone, FromPrimitive)]
#[repr(isize)]
pub enum GitFileMode {
    /// Empty is used as the GitFileMode of tree elements when comparing
    /// trees in the following situations:
    ///
    /// - the mode of tree elements before their creation.  
    /// - the mode of tree elements after their deletion.  
    /// - the mode of unmerged elements when checking the index.
    ///
    /// Empty has no file system equivalent.  As Empty is the zero value
    /// of [GitFileMode]
    Empty = 0,
    /// Regular represent non-executable files.
    Regular = 0o100644,
    /// Dir represent a Directory.
    Dir = 0o40000,
    /// Deprecated represent non-executable files with the group writable bit set.  This mode was
    /// supported by the first versions of git, but it has been deprecated nowadays.  This
    /// library(github.com/go-git/go-git uses it, not realaravinth/gitpad at the moment) uses them
    /// internally, so you can read old packfiles, but will treat them as Regulars when interfacing
    /// with the outside world.  This is the standard git behaviour.
    Deprecated = 0o100664,
    /// Executable represents executable files.
    Executable = 0o100755,
    /// Symlink represents symbolic links to files.
    Symlink = 0o120000,
    /// Submodule represents git submodules.  This mode has no file system
    /// equivalent.
    Submodule = 0o160000,

    /// Unsupported file mode
    #[num_enum(default)]
    Unsupported = -1,
}

impl From<&'_ TreeEntry<'_>> for GitFileMode {
    fn from(t: &TreeEntry) -> Self {
        GitFileMode::from(t.filemode() as isize)
    }
}

impl From<TreeEntry<'_>> for GitFileMode {
    fn from(t: TreeEntry) -> Self {
        GitFileMode::from(t.filemode() as isize)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FileInfo {
    pub filename: String,
    pub content: ContentType,
    pub mime: MimeGuess,
}

#[derive(Serialize, Eq, PartialEq, Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentType {
    Binary(Vec<u8>),
    Text(String),
}

impl ContentType {
    pub fn bytes(self) -> Vec<u8> {
        match self {
            Self::Text(text) => text.into(),
            Self::Binary(bin) => bin,
        }
    }

    pub fn from_blob(blob: &git2::Blob) -> Self {
        if blob.is_binary() {
            Self::Binary(blob.content().to_vec())
        } else {
            Self::Text(String::from_utf8_lossy(blob.content()).to_string())
        }
    }
}

/// Please note that this method expects path to not contain any spaces
/// Use [escape_spaces] before calling this method
///
/// For example, a read request for "foo bar.md" will fail even if that file is present
/// in the repository. However, it will succeed if the output of [escape_spaces] is
/// used in the request.
pub fn read_file(repo_path: &PathBuf, path: &str) -> ServiceResult<FileInfo> {
    let repo = git2::Repository::open(repo_path).unwrap();
    let head = repo.head().unwrap();
    let tree = head.peel_to_tree().unwrap();
    read_file_inner(&repo, path, &tree)
}

pub fn read_preview_file(
    repo_path: &PathBuf,
    preview_name: &str,
    path: &str,
) -> ServiceResult<FileInfo> {
    let repo = git2::Repository::open(repo_path).unwrap();
    let branch = repo
        .find_branch(preview_name, git2::BranchType::Local)
        .unwrap();
    //    let tree = head.peel_to_tree().unwrap();
    let branch = branch.into_reference();
    let tree = branch.peel_to_tree().unwrap();
    read_file_inner(&repo, path, &tree)
}

fn read_file_inner(
    repo: &git2::Repository,
    path: &str,
    tree: &git2::Tree,
) -> ServiceResult<FileInfo> {
    fn read_file(id: Oid, repo: &git2::Repository) -> ContentType {
        let blob = repo.find_blob(id).unwrap();
        ContentType::from_blob(&blob)
    }

    fn get_index_file(id: Oid, repo: &Repository) -> ContentType {
        let tree = repo.find_tree(id).unwrap();
        const INDEX_FILES: [&str; 7] = [
            "index.html",
            "index.md",
            "INDEX.md",
            "README.md",
            "README",
            "readme.txt",
            "readme",
        ];

        let content = if let Some(index_file) = tree.iter().find(|x| {
            if let Some(name) = x.name() {
                INDEX_FILES.iter().any(|index_name| *index_name == name)
            } else {
                false
            }
        }) {
            read_file(index_file.id(), repo)
        } else {
            unimplemented!("Index file not found");
        };
        content
    }

    let inner = |repo: &git2::Repository, tree: &git2::Tree| -> ServiceResult<FileInfo> {
        let mut path = path;
        if path == "/" {
            let content = get_index_file(tree.id(), repo);
            return Ok(FileInfo {
                filename: "/".into(),
                content,
                mime: mime_guess::from_path("index.html"),
            });
        }
        if path.starts_with('/') {
            path = path.trim_start_matches('/');
        }

        fn file_not_found(e: git2::Error) -> ServiceError {
            if e.code() == ErrorCode::NotFound && e.class() == ErrorClass::Tree {
                return ServiceError::FileNotFound;
            }
            e.into()
        }
        let entry = tree.get_path(Path::new(path)).map_err(file_not_found)?;

        let mode: GitFileMode = entry.clone().into();
        if let Some(name) = entry.name() {
            let file = match mode {
                GitFileMode::Dir => get_index_file(entry.id(), repo),
                GitFileMode::Submodule => unimplemented!(),
                GitFileMode::Empty => unimplemented!(),
                GitFileMode::Deprecated => unimplemented!(),
                GitFileMode::Unsupported => unimplemented!(),
                GitFileMode::Symlink => unimplemented!(),
                GitFileMode::Executable => read_file(entry.id(), repo),
                GitFileMode::Regular => read_file(entry.id(), repo),
            };
            Ok(FileInfo {
                filename: name.to_string(),
                mime: mime_guess::from_path(path),
                content: file,
            })
        } else {
            unimplemented!();
        }
    };

    inner(repo, tree)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use mktemp::Temp;

    const FILE_CONTENT: &str = "foobar";

    pub fn write_file_util(repo_path: &str, file_name: &str, content: Option<&str>) {
        // TODO change updated in DB
        let inner = |repo: &mut Repository| -> ServiceResult<()> {
            let mut tree_builder = match repo.head() {
                Err(_) => repo.treebuilder(None).unwrap(),

                Ok(h) => repo.treebuilder(Some(&h.peel_to_tree().unwrap())).unwrap(),
            };

            let odb = repo.odb().unwrap();

            let content = if content.is_some() {
                content.as_ref().unwrap()
            } else {
                FILE_CONTENT
            };

            let obj = odb.write(ObjectType::Blob, content.as_bytes()).unwrap();
            tree_builder.insert(file_name, obj, 0o100644).unwrap();
            let tree_hash = tree_builder.write().unwrap();
            let author = Signature::now("librepages", "admin@librepages.org").unwrap();
            let committer = Signature::now("librepages", "admin@librepages.org").unwrap();

            let commit_tree = repo.find_tree(tree_hash).unwrap();
            let msg = "";
            if let Err(e) = repo.head() {
                if e.code() == ErrorCode::UnbornBranch && e.class() == ErrorClass::Reference {
                    // fisrt commit ever; set parent commit(s) to empty array
                    repo.commit(Some("HEAD"), &author, &committer, msg, &commit_tree, &[])
                        .unwrap();
                } else {
                    panic!("{:?}", e);
                }
            } else {
                let head_ref = repo.head().unwrap();
                let head_commit = head_ref.peel_to_commit().unwrap();
                repo.commit(
                    Some("HEAD"),
                    &author,
                    &committer,
                    msg,
                    &commit_tree,
                    &[&head_commit],
                )
                .unwrap();
            };

            Ok(())
        };

        if Repository::open(repo_path).is_err() {
            let _ = Repository::init(repo_path);
        }
        let mut repo = Repository::open(repo_path).unwrap();
        let _ = inner(&mut repo);
    }

    #[test]
    fn test_git_write_read_works() {
        const FILENAME: &str = "README.txt";

        let tmp_dir = Temp::new_dir().unwrap();
        let path = tmp_dir.to_str().unwrap();

        write_file_util(path, FILENAME, None);
        let resp = read_file(&Path::new(path).into(), FILENAME).unwrap();
        assert_eq!(resp.filename, FILENAME);
        assert_eq!(resp.content.bytes(), FILE_CONTENT.as_bytes());
        assert_eq!(resp.mime.first().unwrap(), "text/plain");

        let resp = read_preview_file(&Path::new(path).into(), "master", FILENAME).unwrap();
        assert_eq!(resp.filename, FILENAME);
        assert_eq!(resp.content.bytes(), FILE_CONTENT.as_bytes());
        assert_eq!(resp.mime.first().unwrap(), "text/plain");

        assert_eq!(
            read_preview_file(&Path::new(path).into(), "master", "file-does-not-exist.txt"),
            Err(ServiceError::FileNotFound)
        );
    }
}
