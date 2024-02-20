use std::path::PathBuf;

use crate::ProxyConfig;

pub fn get_prefetch_path(name: &str, conf: &ProxyConfig) -> PathBuf {
    conf.sparse_dir.join(crate_sub_path(name))
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
