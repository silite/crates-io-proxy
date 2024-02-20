use std::{fs::read, time::Duration};

use once_cell::sync::Lazy;
use poem::{
    get, handler,
    listener::TcpListener,
    trace,
    web::{Data, Json, Path},
    EndpointExt, Route, Server,
};
use serde_json::Value;
use tokio::runtime::{Builder, Runtime};
use url::Url;

use crate::{
    config_json::gen_config_json_file, crate_info::CrateInfo, file_cache::cache_fetch_crate,
    forward_download_request, ProxyConfig,
};

pub static TOKIO_RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Builder::new_multi_thread()
        .thread_name("stats-web")
        .worker_threads(40)
        .enable_all()
        .build()
        .unwrap()
});

pub static ASYNC_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::ClientBuilder::new()
        .connect_timeout(Duration::from_secs(2))
        .timeout(Duration::from_secs(5))
        .user_agent("curl/7.68.0")
        .build()
        .unwrap()
});

pub fn start(conf: ProxyConfig) -> anyhow::Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", 8888));
    let server = Server::new(listener);
    let app = Route::new()
        .at("/index/config.json", get(config))
        .at("/api/v1/crates/:name/:version/download", get(download))
        .at("/index/:a/:b/:name", get(prefetch_crates))
        .at("/index/:a/:name", get(prefetch_len2_crates))
        .data(conf);
    TOKIO_RUNTIME.block_on(server.run(app))?;
    Ok(())
}

#[handler]
fn config(Data(conf): Data<&ProxyConfig>) -> Json<Value> {
    Json(serde_json::from_str(&gen_config_json_file(conf)).unwrap())
}

#[handler]
fn download(
    Path((name, version)): Path<(String, String)>,
    Data(conf): Data<&ProxyConfig>,
) -> Vec<u8> {
    let crate_info = CrateInfo::new(&name, &version);
    if let Some(data) = cache_fetch_crate(&conf.crates_dir, &crate_info) {
        data
    } else {
        forward_download_request(crate_info, conf.clone())
    }
}

#[handler]
async fn prefetch_crates(
    Path((_a, _b, name)): Path<(String, String, String)>,
    Data(conf): Data<&ProxyConfig>,
) -> Vec<u8> {
    prefetch_with_name(&name, conf).await
}

#[handler]
async fn prefetch_len2_crates(
    Path((_a, name)): Path<(String, String)>,
    Data(conf): Data<&ProxyConfig>,
) -> Vec<u8> {
    prefetch_with_name(&name, conf).await
}

async fn prefetch_with_name(name: &str, conf: &ProxyConfig) -> Vec<u8> {
    trace!("{:?}", conf.sparse_dir.join(crate_sub_path(name)));
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
