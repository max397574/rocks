use eyre::eyre;
use eyre::Result;
use git2::Repository;
///! Retrieves metadata from a given github repository
use std::path::PathBuf;

#[derive(Debug)]
pub struct RepoMetadata {
    pub name: String,
    pub license: Option<String>,
    pub labels: Option<Vec<String>>,
    pub contributors: Vec<String>,
}

/// Retrieves metadata for a given directory
pub async fn get_metadata_for(directory: Option<&PathBuf>) -> Result<Option<RepoMetadata>> {
    let repo = match directory {
        Some(path) => Repository::discover(path)?,
        None => Repository::open_from_env()?,
    };

    // NOTE(vhyrro): Temporary value is required. Thank the borrow checker.
    let ret = if let Some(remote) = repo.find_remote("origin")?.url() {
        let parsed_url = git_url_parse::GitUrl::parse(remote)?;

        let (owner, name) = match (parsed_url.owner, parsed_url.name) {
            (Some(owner), name) => (owner, name),
            _ => return Err(eyre!("unable to parse remote `origin` - it's likely that your upstream remote is malformed!")),
        };

        let octocrab = octocrab::instance();
        let repo_handler = octocrab.repos(owner, name);

        let contributors = repo_handler.list_contributors().send().await?;
        let repo_data = repo_handler.get().await?;

        Ok(Some(RepoMetadata {
            name: repo_data.name,
            license: repo_data.license.map(|license| license.name),
            labels: repo_data.topics,
            contributors: contributors
                .items
                .into_iter()
                .map(|contributor| contributor.author.login)
                .collect(),
        }))
    } else {
        Ok(None)
    };

    ret
}