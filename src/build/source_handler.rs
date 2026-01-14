use git2::Repository;

use crate::build::package::Source;

pub struct GitSourceHandler {
    source: Source,
    path: String,
}

pub trait SourceHandler {
    fn pull_source(&self) -> Result<(), &'static str>;
}

impl GitSourceHandler {
    pub fn new(source: Source) -> Result<Self, &'static str> {
        match &source {
            Source::Git { repo, .. } => Ok(GitSourceHandler {
                source,
                path: "/tmp/".to_string(),
            }),
            _ => Err("GitSourceHandler only accepts Git sources"),
        }
    }
}

// TODO
// Allow for clone and pull 
impl SourceHandler for GitSourceHandler {
    fn pull_source(&self) -> Result<(), &'static str> {
        // Destructure Git source
        let Source::Git {
            repo,
            tag,
            branch,
            commit,
        } = &self.source;

        std::fs::create_dir_all(&self.path).map_err(|_| "Failed to create path")?;

        let git_repo =
            Repository::clone(repo, &self.path).map_err(|_| "Failed to clone repository")?;

        if let Some(commit) = commit {
            let oid = git2::Oid::from_str(commit).map_err(|_| "Invalid commit hash")?;
            let obj = git_repo
                .find_object(oid, None)
                .map_err(|_| "Commit not found")?;
            git_repo
                .checkout_tree(&obj, None)
                .map_err(|_| "Failed to checkout commit")?;
            git_repo
                .set_head_detached(oid)
                .map_err(|_| "Failed to detach HEAD")?;
        } else if let Some(tag) = tag {
            let refname = format!("refs/tags/{tag}");
            let obj = git_repo
                .revparse_single(&refname)
                .map_err(|_| "Tag not found")?;
            git_repo
                .checkout_tree(&obj, None)
                .map_err(|_| "Failed to checkout tag")?;
            git_repo
                .set_head(&refname)
                .map_err(|_| "Failed to set HEAD to tag")?;
        } else if let Some(branch) = branch {
            let refname = format!("refs/remotes/origin/{branch}");
            let obj = git_repo
                .revparse_single(&refname)
                .map_err(|_| "Branch not found")?;
            git_repo
                .checkout_tree(&obj, None)
                .map_err(|_| "Failed to checkout branch")?;
            git_repo
                .set_head(&refname)
                .map_err(|_| "Failed to set HEAD to branch")?;
        }

        Ok(())
    }
}
