use std::io::Read;

use md5::Md5;
use sha2::{Digest, Sha512};

use crate::error::Error;

pub struct DownloadHashes {
    pub md5: String,
    pub sha512: String,
}

/// # Errors
/// Returns an error if the HTTP client cannot be constructed.
pub fn build_client() -> Result<reqwest::blocking::Client, Error> {
    Ok(reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_mins(10))
        .build()?)
}

/// # Errors
/// Returns an error on network failure or non-success HTTP status.
pub fn download_body(client: &reqwest::blocking::Client, url: &str) -> Result<Vec<u8>, Error> {
    let resp = client.get(url).send()?;

    if !resp.status().is_success() {
        return Err(Error::HttpStatus {
            url: url.to_string(),
            status: resp.status(),
        });
    }

    Ok(resp.bytes()?.to_vec())
}

/// # Errors
/// Returns an error on network failure or non-success HTTP status.
pub fn download_and_hash(client: &reqwest::blocking::Client, url: &str) -> Result<DownloadHashes, Error> {
    let mut resp = client.get(url).send()?;

    if !resp.status().is_success() {
        return Err(Error::HttpStatus {
            url: url.to_string(),
            status: resp.status(),
        });
    }

    let mut sha512_hasher = Sha512::new();
    let mut md5_hasher = Md5::new();
    let mut buf = vec![0u8; 64 * 1024];

    loop {
        let n = resp.read(&mut buf)?;
        if n == 0 {
            break;
        }
        sha512_hasher.update(&buf[..n]);
        md5_hasher.update(&buf[..n]);
    }

    Ok(DownloadHashes {
        md5: hex::encode(md5_hasher.finalize()),
        sha512: hex::encode(sha512_hasher.finalize()),
    })
}
