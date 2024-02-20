use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder, Result};
use once_cell::sync::Lazy;
use serde_json::Value;
use tokio::runtime::{Builder, Runtime};

use crate::{
    config_json::gen_config_json_file, crate_info::CrateInfo, file_cache::cache_fetch_crate,
    forward_download_request, init::prefetch_with_name, ProxyConfig,
};

// pub fn start(conf: ProxyConfig) {
//     let app = Router::new()
//         .route("/index/config.json", get(config))
//         .route("/api/v1/crates/:name/:version/download", get(download))
//         .route("/index/:a/:b/:name", get(prefetch_crates))
//         .route("/index/:a/:name", get(prefetch_len2_crates))
//         .with_state(conf);

#[actix_web::main]
pub async fn start(conf: ProxyConfig) {
    let _ = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(conf.clone()))
            .route("/index/config.json", web::get().to(config))
            .service(prefetch_crates)
            .service(prefetch_len2_crates)
    })
    .bind(("0.0.0.0", 8888))
    .unwrap()
    .run()
    .await;
}

async fn config(conf: web::Data<ProxyConfig>) -> web::Json<Value> {
    let conf = serde_json::from_str::<Value>(&gen_config_json_file(&conf)).unwrap();
    web::Json(conf)
}

// async fn download(
//     Path((name, version)): Path<(String, String)>,
//     State(conf): State<ProxyConfig>,
// ) -> Vec<u8> {
//     let crate_info = CrateInfo::new(&name, &version);
//     if let Some(data) = cache_fetch_crate(&conf.crates_dir, &crate_info) {
//         data
//     } else {
//         forward_download_request(crate_info, conf.clone())
//     }
// }

#[get("/index/{_a}/{_b}/{name}")]
async fn prefetch_crates(req: HttpRequest, conf: web::Data<ProxyConfig>) -> Vec<u8> {
    let name = req.match_info().get("name").unwrap();
    prefetch_with_name(&name, &conf).await
}

#[get("/index/{_a}/{name}")]
async fn prefetch_len2_crates(req: HttpRequest, conf: web::Data<ProxyConfig>) -> Vec<u8> {
    let name = req.match_info().get("name").unwrap();
    prefetch_with_name(&name, &conf).await
}
