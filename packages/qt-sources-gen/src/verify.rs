use std::collections::HashMap;

use tracing::{debug, error, info, warn};

use crate::error::Error;
use crate::{client, modules};

/// Result of verifying/hashing a single module.
pub struct ModuleHash {
    pub filename: String,
    pub sha512: String,
}

/// Download all module archives, compute SHA-512 + MD5, and verify MD5 against
/// the expected values from `md5sums.txt`. Optionally verify SHA-512 against
/// previously stored hashes.
///
/// The module list is derived dynamically from the filenames in `expected_md5s`
/// rather than a hardcoded list.
///
/// Returns the ordered list of (filename, sha512) pairs for all modules on success.
///
/// # Errors
/// Returns an error if any module fails to download or its hashes do not match.
pub fn download_and_verify<S: ::std::hash::BuildHasher>(
    client: &reqwest::blocking::Client,
    version: &str,
    expected_md5s: &HashMap<String, String, S>,
    stored_sha512s: Option<&HashMap<String, String, S>>,
) -> Result<Vec<ModuleHash>, Error> {
    let (major_minor, _) = modules::split_version(version).ok_or_else(|| Error::InvalidVersion(version.to_string()))?;

    let filenames = modules::archive_filenames(expected_md5s);
    let mut results = Vec::with_capacity(filenames.len());
    let mut failures: Vec<Error> = Vec::new();

    for filename in &filenames {
        let module = modules::module_name(filename);
        let url = modules::archive_url(major_minor, version, filename);
        debug!(module, %url, "downloading");

        match client::download_and_hash(client, &url) {
            Ok(hashes) => {
                // Verify MD5 against md5sums.txt
                if let Some(expected_md5) = expected_md5s.get(filename.as_str()) {
                    if hashes.md5 != *expected_md5 {
                        error!(module, expected = %expected_md5, actual = %hashes.md5, "MD5 mismatch");
                        failures.push(Error::Md5Mismatch {
                            filename: filename.clone(),
                            expected: expected_md5.clone(),
                            actual: hashes.md5,
                        });
                        continue;
                    }
                    debug!(module, "MD5 verified");
                } else {
                    warn!(module, "no MD5 entry in md5sums.txt — skipping MD5 verification");
                }

                // Optionally verify SHA-512 against stored value
                if let Some(stored) = stored_sha512s {
                    if let Some(stored_sha512) = stored.get(filename.as_str()) {
                        if hashes.sha512 != *stored_sha512 {
                            error!(module, expected = %stored_sha512, actual = %hashes.sha512, "SHA-512 mismatch");
                            failures.push(Error::Sha512Mismatch {
                                filename: filename.clone(),
                            });
                            continue;
                        }
                        debug!(module, "SHA-512 verified");
                    } else {
                        warn!(module, "no stored SHA-512 — cannot verify");
                    }
                }

                info!(module, "ok");
                results.push(ModuleHash {
                    filename: filename.clone(),
                    sha512: hashes.sha512,
                });
            }
            Err(e) => {
                error!(module, err = %e, "download failed");
                failures.push(e);
            }
        }
    }

    if !failures.is_empty() {
        let count = failures.len();
        warn!(count, "some modules failed");
        for f in &failures {
            error!(failure = %f);
        }
        return Err(Error::VerificationFailed { count });
    }

    Ok(results)
}
