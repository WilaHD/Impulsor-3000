use std::{cmp::Ordering, io::Read, time::Duration};

use semver::Version;
use serde::Deserialize;

const LATEST_RELEASE_API: &str =
    "https://api.github.com/repos/WilaHD/Impulsor-3000/releases/latest";
pub const RELEASES_URL: &str = "https://github.com/WilaHD/Impulsor-3000/releases/latest";
const MAX_RESPONSE_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    prerelease: bool,
    draft: bool,
}

pub fn check() -> Option<UpdateInfo> {
    check_url(LATEST_RELEASE_API, env!("IMPULSOR_APP_VERSION")).ok()?
}

fn check_url(url: &str, current_version: &str) -> Result<Option<UpdateInfo>, String> {
    let response = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(8))
        .build()
        .get(url)
        .set("User-Agent", "Impulsor-3000-update-check")
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|error| error.to_string())?;
    let mut body = Vec::new();
    response
        .into_reader()
        .take(MAX_RESPONSE_BYTES + 1)
        .read_to_end(&mut body)
        .map_err(|error| error.to_string())?;
    if body.len() as u64 > MAX_RESPONSE_BYTES {
        return Err("Release response is too large".into());
    }
    parse_release(&body, current_version)
}

fn parse_release(body: &[u8], current_version: &str) -> Result<Option<UpdateInfo>, String> {
    let release: Release = serde_json::from_slice(body).map_err(|error| error.to_string())?;
    let current = parse_version(current_version)?;
    let latest = parse_version(&release.tag_name)?;
    if release.prerelease
        || release.draft
        || !latest.pre.is_empty()
        || latest.cmp_precedence(&current) != Ordering::Greater
    {
        return Ok(None);
    }
    Ok(Some(UpdateInfo {
        current_version: current_version.to_owned(),
        latest_version: release.tag_name,
    }))
}

fn parse_version(version: &str) -> Result<Version, String> {
    Version::parse(version.strip_prefix('v').unwrap_or(version)).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };

    fn release(tag: &str, prerelease: bool, draft: bool) -> Vec<u8> {
        serde_json::json!({"tag_name": tag, "prerelease": prerelease, "draft": draft})
            .to_string()
            .into_bytes()
    }

    #[test]
    fn compares_semantic_versions_and_prereleases() {
        assert!(
            parse_release(&release("v1.0.0+new", false, false), "1.0.0+old")
                .unwrap()
                .is_none()
        );
        assert!(parse_release(&release("v1.10.0", false, false), "1.9.0")
            .unwrap()
            .is_some());
        assert!(
            parse_release(&release("v1.0.0", false, false), "1.0.0-beta.1")
                .unwrap()
                .is_some()
        );
        assert!(parse_release(&release("v1.0.0", false, false), "v1.0.0")
            .unwrap()
            .is_none());
        assert!(parse_release(&release("v1.0.0", false, false), "1.1.0")
            .unwrap()
            .is_none());
        assert!(
            parse_release(&release("v2.0.0-beta.1", true, false), "1.0.0")
                .unwrap()
                .is_none()
        );
        assert!(
            parse_release(&release("v2.0.0-beta.1", false, false), "1.0.0")
                .unwrap()
                .is_none()
        );
        assert!(parse_release(&release("v2.0.0", false, true), "1.0.0")
            .unwrap()
            .is_none());
    }

    #[test]
    fn rejects_invalid_responses_and_versions() {
        assert!(parse_release(b"not json", "1.0.0").is_err());
        assert!(parse_release(br#"{"tag_name":"v2.0.0"}"#, "1.0.0").is_err());
        assert!(parse_release(&release("latest", false, false), "1.0.0").is_err());
        assert!(parse_release(&release("v2.0.0", false, false), "invalid").is_err());
    }

    fn serve_once(status: &str, body: &[u8]) -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        let body = body.to_vec();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 4096];
            stream.read(&mut request).unwrap();
            stream.write_all(response.as_bytes()).unwrap();
            stream.write_all(&body).unwrap();
        });
        (format!("http://{address}/latest"), handle)
    }

    #[test]
    fn checks_http_responses_without_external_network() {
        let (url, server) = serve_once("200 OK", &release("v2.0.0", false, false));
        assert_eq!(
            check_url(&url, "1.0.0").unwrap().unwrap().latest_version,
            "v2.0.0"
        );
        server.join().unwrap();
        let (url, server) = serve_once("403 Forbidden", b"rate limited");
        assert!(check_url(&url, "1.0.0").is_err());
        server.join().unwrap();
    }
}
