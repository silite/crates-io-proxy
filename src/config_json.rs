//! Sparse registry configuration file helpers

use log::trace;

use super::{ProxyConfig, CRATES_API_PATH};

/// Dynamically generates the registry configuration file contents.
#[must_use]
pub(super) fn gen_config_json_file(config: &ProxyConfig) -> String {
    // Generate the crate download API URL pointing to this same proxy server.
    let dl_url = config
        .proxy_url
        .join(CRATES_API_PATH)
        .expect("invalid proxy server URL");

    // Cargo can not handle trailing slashes in `config.json`.
    let dl = dl_url.as_str().trim_end_matches('/');
    let api = config.upstream_url.as_str().trim_end_matches('/');
    trace!("config.json: dl={}, api={}", dl, api);

    format!(r#"{{"dl":"{dl}","api":"{api}"}}"#)
}
