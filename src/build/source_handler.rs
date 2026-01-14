use git2::{Oid, Repository};
use tempfile::TempDir;
use thiserror::Error;

use crate::build::package::Source;

#[derive(Debug, Error)]
pub enum GitSourceHandlerError {
    #[error("Failed to create temp dir: {0}")]
    FailedCreatingTempDirError(#[from] std::io::Error),

    #[error("Failed to clone git repo: {0}")]
    FailedFetchingGitRepoError(#[from] git2::Error),

    #[error("Unsupported source type: {0}")]
    InvalidSpecifications(String),

    #[error("Unsupported source type")]
    Unsupported,
}

#[derive(Debug)]
pub struct GitSource<'a> {
    pub repo: &'a str,
    pub tag: Option<&'a str>,
    pub branch: Option<&'a str>,
    pub commit: Option<&'a str>,
}

impl<'a> GitSource<'a> {
    pub fn from_source(source: &'a Source) -> Result<Self, GitSourceHandlerError> {
        if let Source::Git {
            repo,
            tag,
            branch,
            commit,
        } = source
        {
            let choices = [commit.is_some(), tag.is_some(), branch.is_some()];

            if choices.iter().filter(|&&x| x).count() > 1 {
                return Err(GitSourceHandlerError::InvalidSpecifications(
                    "Only one of commit, tag, or branch may be specified".into(),
                ));
            }

            Ok(Self {
                repo,
                tag: tag.as_deref(),
                branch: branch.as_deref(),
                commit: commit.as_deref(),
            })
        } else {
            Err(GitSourceHandlerError::Unsupported)
        }
    }
}

pub struct GitSourceHandler<'a> {
    source: GitSource<'a>,
}

impl<'a> GitSourceHandler<'a> {
    pub fn new(source: GitSource<'a>) -> Self {
        Self { source }
    }

    pub fn fetch(&self) -> Result<TempDir, GitSourceHandlerError> {
        let dir = TempDir::new()?;

        let repo = Repository::clone(self.source.repo, dir.path())?;

        let object = match (self.source.commit, self.source.tag, self.source.branch) {
            (Some(commit), _, _) => {
                let oid = Oid::from_str(commit)?;
                repo.find_object(oid, None)?
            }
            (None, Some(tag), _) => repo.revparse_single(&format!("refs/tags/{tag}"))?,
            (None, None, Some(branch)) => repo.revparse_single(&format!("refs/heads/{branch}"))?,
            (None, None, None) => repo.head()?.peel(git2::ObjectType::Commit)?,
        };

        let mut checkout_opts = git2::build::CheckoutBuilder::new();
        checkout_opts.force();
        repo.checkout_tree(&object, Some(&mut checkout_opts))?;

        Ok(dir)
    }
}
