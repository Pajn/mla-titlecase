use serde::Deserialize;

use crate::error::{CliError, Result};

pub(crate) const USER_AGENT: &str = concat!("mla-titlecase-cli/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, Clone)]
pub(crate) struct GithubFile {
    pub(crate) download_url: String,
    pub(crate) html_url: String,
    pub(crate) sha: String,
}

#[derive(Debug, Deserialize)]
struct GithubContentResponse {
    download_url: Option<String>,
    html_url: String,
}

#[derive(Debug, Deserialize)]
struct GithubCommitResponse {
    sha: String,
}

pub(crate) fn client() -> Result<reqwest::blocking::Client> {
    Ok(reqwest::blocking::Client::builder().user_agent(USER_AGENT).build()?)
}

pub(crate) fn resolve_file(
    client: &reqwest::blocking::Client,
    owner: &str,
    repo: &str,
    path: &str,
    reference: &str,
) -> Result<GithubFile> {
    let url =
        format!("https://api.github.com/repos/{owner}/{repo}/contents/{path}?ref={reference}");
    let response: GithubContentResponse = client.get(url).send()?.error_for_status()?.json()?;
    let _download_url = response.download_url.ok_or_else(|| {
        CliError::SourceMetadata(format!(
            "GitHub contents API did not expose a raw download URL for {owner}/{repo}:{path}"
        ))
    })?;

    let commit_url = format!(
        "https://api.github.com/repos/{owner}/{repo}/commits?path={path}&sha={reference}&per_page=1"
    );
    let commit = client
        .get(commit_url)
        .send()?
        .error_for_status()?
        .json::<Vec<GithubCommitResponse>>()?
        .into_iter()
        .next()
        .ok_or_else(|| {
            CliError::SourceMetadata(format!(
                "GitHub commits API returned no commit for {owner}/{repo}:{path}@{reference}"
            ))
        })?;

    let download_url =
        format!("https://raw.githubusercontent.com/{owner}/{repo}/{}/{path}", commit.sha);

    Ok(GithubFile { download_url, html_url: response.html_url, sha: commit.sha })
}

pub(crate) fn download_bytes(client: &reqwest::blocking::Client, url: &str) -> Result<Vec<u8>> {
    Ok(client.get(url).send()?.error_for_status()?.bytes()?.to_vec())
}

pub(crate) fn download_text(client: &reqwest::blocking::Client, url: &str) -> Result<String> {
    Ok(String::from_utf8(download_bytes(client, url)?)?)
}
