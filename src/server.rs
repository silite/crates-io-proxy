use actix_files as fs;
use actix_web::{get, web, App, Error, HttpRequest, HttpServer, Result};
use serde_json::Value;

use crate::{
    config_json::gen_config_json_file, crate_info::CrateInfo, forward_download_request,
    init::get_prefetch_path, ProxyConfig,
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
            .service(download)
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

#[get("/api/v1/crates/{name}/{version}/download")]
async fn download(req: HttpRequest, conf: web::Data<ProxyConfig>) -> Result<fs::NamedFile, Error> {
    let name = req.match_info().get("name").unwrap();
    let version = req.match_info().get("version").unwrap();
    let crate_info = CrateInfo::new(&name, &version);

    let path = conf.crates_dir.join(crate_info.to_file_path());
    if !path.is_file() {
        forward_download_request(crate_info, &conf)
    }
    let file = fs::NamedFile::open(path)?;
    return Ok(file.use_last_modified(true).use_etag(true));
}

#[get("/index/{_a}/{_b}/{name}")]
async fn prefetch_crates(
    req: HttpRequest,
    conf: web::Data<ProxyConfig>,
) -> Result<fs::NamedFile, Error> {
    let name = req.match_info().get("name").unwrap();
    let file = fs::NamedFile::open(get_prefetch_path(name, &conf))?;
    Ok(file.use_last_modified(true).use_etag(true))
}

#[get("/index/{_a}/{name}")]
async fn prefetch_len2_crates(
    req: HttpRequest,
    conf: web::Data<ProxyConfig>,
) -> Result<fs::NamedFile, Error> {
    let name = req.match_info().get("name").unwrap();
    let file = fs::NamedFile::open(get_prefetch_path(name, &conf))?;
    Ok(file.use_last_modified(true).use_etag(true))
}
