use once_cell::sync::Lazy;
use poem::{
    get, handler,
    listener::TcpListener,
    web::{Data, Path},
    EndpointExt, Route, Server,
};
use tokio::runtime::{Builder, Runtime};

use crate::{
    config_json::gen_config_json_file,
    crate_info::CrateInfo,
    file_cache::cache_fetch_crate,
    forward_download_request,
    resp::{ApiResponseOk, FtHttpResponse},
    send_crate_data_response, ProxyConfig,
};

pub static TOKIO_RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    Builder::new_multi_thread()
        .thread_name("stats-web")
        .worker_threads(40)
        .enable_all()
        .build()
        .unwrap()
});

pub fn start(conf: ProxyConfig) -> anyhow::Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", 8888));
    let server = Server::new(listener);
    let app = Route::new()
        .at("/index/config.json", get(config))
        .at("/:name/:version/download", get(download))
        .at("/:a/:b/:name", get(prefetch_crates))
        .at("/:a/:name", get(prefetch_len2_crates))
        .data(conf);
    TOKIO_RUNTIME.block_on(server.run(app))?;
    Ok(())
}

#[handler]
fn config(Data(conf): Data<&ProxyConfig>) -> FtHttpResponse {
    gen_config_json_file(conf).json_ok()
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
fn prefetch_crates() {}

#[handler]
fn prefetch_len2_crates() {}
