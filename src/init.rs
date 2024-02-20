use std::{collections::BTreeMap, fs::read, path::PathBuf};

use crate::ProxyConfig;

static mut PREFETCH_JSON: BTreeMap<PathBuf, Vec<u8>> = BTreeMap::new();
pub async fn prefetch_with_name(name: &str, conf: &ProxyConfig) -> Vec<u8> {
    let path = conf.sparse_dir.join(crate_sub_path(name));
    unsafe {
        if let Some(res) = PREFETCH_JSON.get_mut(&path) {
            res.clone()
        } else {
            let file = read(path.clone()).unwrap();
            PREFETCH_JSON.insert(path, file.clone());
            return file;
        }
    }
}

pub fn crate_sub_path(name: &str) -> String {
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
