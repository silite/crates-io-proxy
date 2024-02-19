use poem::web::Json;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone)]
pub struct Pagination {
    pub page: u32,
    pub size: u32,
}

pub type FtHttpResponse = Json<serde_json::Value>;

#[derive(Serialize)]
pub struct JsonReturn<T: serde::Serialize> {
    pub result: T,
    pub status: String,
}

pub fn json_return<T: serde::Serialize>(result: T, status: String) -> Json<serde_json::Value> {
    let json_return = JsonReturn { result, status };
    Json(serde_json::to_value(&json_return).unwrap())
}

pub trait ApiResponseOk {
    fn json_ok(self) -> Json<serde_json::Value>;
}

impl<T: serde::Serialize> ApiResponseOk for T {
    fn json_ok(self) -> Json<serde_json::Value> {
        json_return(self, "ok".to_string())
    }
}

pub trait ApiResponseResult {
    fn json(self) -> FtHttpResponse;
}

impl<T: serde::Serialize> ApiResponseResult for anyhow::Result<T> {
    fn json(self) -> FtHttpResponse {
        match self {
            Err(err) => err.json_err(),
            Ok(d) => d.json_ok(),
        }
    }
}

pub trait ApiResponseErr {
    fn json_err(self) -> Json<serde_json::Value>;
}

impl ApiResponseErr for &str {
    fn json_err(self) -> Json<serde_json::Value> {
        json_return(self, "err".to_owned())
    }
}

impl ApiResponseErr for String {
    fn json_err(self) -> Json<serde_json::Value> {
        json_return(self, "err".to_owned())
    }
}

impl ApiResponseErr for anyhow::Error {
    fn json_err(self) -> Json<serde_json::Value> {
        format!("{}", self).json_err()
    }
}

pub trait SqlError {
    fn json_sql_err(self) -> Json<serde_json::Value>;
}

#[derive(Deserialize, Clone)]
pub struct PaginationResp<T> {
    pub params: Option<T>,
    pub pagination: Option<Pagination>,
}
