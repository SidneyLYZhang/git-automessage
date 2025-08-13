//     _         _        __  __
//    / \  _   _| |_ ___ |  \/  | ___  ___ ___  __ _  __ _  ___
//   / _ \| | | | __/ _ \| |\/| |/ _ \/ __/ __|/ _` |/ _` |/ _ \
//  / ___ \ |_| | || (_) | |  | |  __/\__ \__ \ (_| | (_| |  __/
// /_/   \_\__,_|\__\___/|_|  |_|\___||___/___/\__,_|\__, |\___|
//                                                   |___/
//
// Author: Sidney Zhang <zly@lyzhang.me>
// Date: 2025-08-05
// License: MIT
//
// A git repository wrapper.

use anyhow::{Context, Result};
use git2::{Diff, DiffOptions, Repository, Signature};
use std::path::Path;

#[derive(Debug)]
pub struct StagedFile {
    pub path: String,
    pub status: String,
}

#[derive(Debug)]
pub struct CommitInfo {
    pub sha: String,
    pub message: String,
    pub author: String,
    pub date: String,
    pub files_changed: Vec<String>,
}

pub struct GitRepo {
    repo: Repository,
}

impl GitRepo {
    pub fn open() -> Result<Self> {
        let repo = Repository::open(".")?;
        Ok(GitRepo { repo })
    }

    pub fn get_staged_files(&self) -> Result<Vec<StagedFile>> {
        let mut diff_opts = DiffOptions::new();
        diff_opts.include_untracked(true);
        
        let diff = self.repo.diff_index_to_workdir(None, Some(&mut diff_opts))?;
        let mut files = Vec::new();

        diff.foreach(
            &mut |delta, _| {
                if let Some(path) = delta.new_file().path() {
                    let status = match delta.status() {
                        git2::Delta::Added => "added",
                        git2::Delta::Modified => "modified",
                        git2::Delta::Deleted => "deleted",
                        git2::Delta::Renamed => "renamed",
                        git2::Delta::Copied => "copied",
                        _ => "unknown",
                    };
                    files.push(StagedFile {
                        path: path.to_string_lossy().to_string(),
                        status: status.to_string(),
                    });
                }
                true
            },
            None,
            None,
            None,
        )?;

        Ok(files)
    }

    pub fn get_staged_diff(&self) -> Result<String> {
        let mut diff_opts = DiffOptions::new();
        diff_opts.include_untracked(true);
        
        let diff = self.repo.diff_index_to_workdir(None, Some(&mut diff_opts))?;
        let mut diff_text = String::new();
        
        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            let prefix = match line.origin() {
                '+' => "+",
                '-' => "-",
                ' ' => " ",
                _ => "",
            };
            diff_text.push_str(&format!("{}{}", prefix, std::str::from_utf8(line.content()).unwrap_or("")));
            true
        })?;

        Ok(diff_text)
    }

    pub fn get_commit_info(&self, reference: &str) -> Result<CommitInfo> {
        let obj = self.repo.revparse_single(reference)?;
        let commit = obj.peel_to_commit()?;
        
        let sha = commit.id().to_string();
        let message = commit.message().unwrap_or("").to_string();
        let author = commit.author().name().unwrap_or("Unknown").to_string();
        let date = format!("{}", commit.time().seconds());
        
        let tree = commit.tree()?;
        let parent = commit.parent(0).ok();
        let parent_tree = parent.as_ref().map(|p| p.tree().ok()).flatten();
        
        let diff = self.repo.diff_tree_to_tree(
            parent_tree.as_ref(),
            Some(&tree),
            None
        )?;
        
        let mut files_changed = Vec::new();
        diff.foreach(
            &mut |delta, _| {
                if let Some(path) = delta.new_file().path() {
                    files_changed.push(path.to_string_lossy().to_string());
                }
                true
            },
            None,
            None,
            None,
        )?;

        Ok(CommitInfo {
            sha,
            message,
            author,
            date,
            files_changed,
        })
    }

    pub fn get_recent_commits(&self, count: usize) -> Result<Vec<CommitInfo>> {
        let mut revwalk = self.repo.revwalk()?;
        revwalk.push_head()?;
        
        let mut commits = Vec::new();
        for oid in revwalk.take(count) {
            let oid = oid?;
            let _commit = self.repo.find_commit(oid)?;
            
            let info = self.get_commit_info(&oid.to_string())?;
            commits.push(info);
        }
        
        Ok(commits)
    }

    pub fn get_commits_in_range(&self, range: &str) -> Result<Vec<CommitInfo>> {
        let range_parts: Vec<&str> = range.split("..").collect();
        if range_parts.len() != 2 {
            anyhow::bail!("Invalid range format. Use format: start..end");
        }

        let start_commit = self.repo.revparse_single(range_parts[0])?;
        let end_commit = self.repo.revparse_single(range_parts[1])?;

        let mut revwalk = self.repo.revwalk()?;
        revwalk.push(end_commit.id())?;
        revwalk.hide(start_commit.id())?;

        let mut commits = Vec::new();
        for oid in revwalk {
            let oid = oid?;
            let info = self.get_commit_info(&oid.to_string())?;
            commits.push(info);
        }

        Ok(commits)
    }

    pub fn create_commit(&self, message: &str) -> Result<()> {
        let signature = Signature::now("Git AutoMessage", "automessage@git")?;
        let tree = self.repo.find_tree(self.repo.index()?.write_tree()?)?;
        
        let parent_commit = self.repo.head()?.peel_to_commit()?;
        
        self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parent_commit],
        )?;

        Ok(())
    }

    pub fn create_annotated_tag(&self, name: &str, message: &str, reference: &str) -> Result<()> {
        let obj = self.repo.revparse_single(reference)?;
        let commit = obj.peel_to_commit()?;
        
        let signature = Signature::now("Git AutoMessage", "automessage@git")?;
        
        let object = commit.as_object();
        self.repo.tag(
            name,
            object,
            &signature,
            message,
            false,
        )?;

        Ok(())
    }
}