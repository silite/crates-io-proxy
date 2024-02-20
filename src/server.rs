use std::fs::read;

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use once_cell::sync::Lazy;
use serde_json::Value;
use tokio::runtime::{Builder, Runtime};

use crate::{
    config_json::gen_config_json_file, crate_info::CrateInfo, file_cache::cache_fetch_crate,
    forward_download_request, ProxyConfig,
};

pub static TOKIO_RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Builder::new_multi_thread()
        .thread_name("stats-web")
        .worker_threads(64)
        .enable_all()
        .enable_io()
        .build()
        .unwrap()
});

pub fn start(conf: ProxyConfig) {
    let app = Router::new()
        .route("/index/config.json", get(config))
        .route("/api/v1/crates/:name/:version/download", get(download))
        .route("/index/:a/:b/:name", get(prefetch_crates))
        .route("/index/:a/:name", get(prefetch_len2_crates))
        .with_state(conf);

    TOKIO_RUNTIME.block_on(async {
        let listener = tokio::net::TcpListener::bind("0.0.0.0:8888").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });
}

async fn config(State(conf): State<ProxyConfig>) -> Json<Value> {
    Json(serde_json::from_str(&gen_config_json_file(&conf)).unwrap())
}

async fn download(
    Path((name, version)): Path<(String, String)>,
    State(conf): State<ProxyConfig>,
) -> Vec<u8> {
    let crate_info = CrateInfo::new(&name, &version);
    if let Some(data) = cache_fetch_crate(&conf.crates_dir, &crate_info) {
        data
    } else {
        forward_download_request(crate_info, conf.clone())
    }
}

async fn prefetch_crates(
    Path((_a, _b, name)): Path<(String, String, String)>,
    State(conf): State<ProxyConfig>,
) -> Vec<u8> {
    prefetch_with_name(&name, &conf).await
}

async fn prefetch_len2_crates(
    Path((_a, name)): Path<(String, String)>,
    State(conf): State<ProxyConfig>,
) -> Vec<u8> {
    prefetch_with_name(&name, &conf).await
}

async fn prefetch_with_name(name: &str, conf: &ProxyConfig) -> Vec<u8> {
    read(conf.sparse_dir.join(crate_sub_path(name))).unwrap()
}
fn crate_sub_path(name: &str) -> String {
    match name.len() {
        1 => format!("1/{}", name),
        2 => format!("2/{}", name),
        3 => {
            let first_char = &name[0..1];
            format!("3/{}/{}", first_char, name)
        }
        _ => {
            let first_two = &name[0..2];
            let second_two = &name[2..4];
            format!("{}/{}/{}", first_two, second_two, name)
        }
    }
}
