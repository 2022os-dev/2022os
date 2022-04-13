use super::*;
use alloc::vec::Vec;

pub fn split_path<'c>(path: &'c str) -> (&'c str, Option<&'c str>) {
    let mut path_split = path.trim_matches('/').splitn(2, '/');
    let comp = path_split.next().unwrap(); // SAFE: splitn always returns at least one element
    let rest_opt = path_split.next();
    (comp, rest_opt)
}

pub fn rsplit_path<'c>(path: &'c str) -> (Option<&'c str>, &'c str) {
    let mut path_split = path.trim_end_matches('/').rsplitn(2, '/');
    let comp = path_split.next().unwrap(); // SAFE: splitn always returns at least one element
    let rest_opt = path_split.next();
    (rest_opt, comp)
}

pub fn parse_path(from: &Inode, path: &str) -> Result<Inode, FileErr> {
    log!("path_resolve":"{}">"", path);
    let mut nodes = Vec::new();
    nodes.push(from.clone());
    let mut rest = Some(path);
    while let Some(rest_path) = rest {
        if let Some(inode) = nodes.last() {
            let (name, mut _rest) = split_path(rest_path);
            rest = _rest;

            log!("path_resolve":>"item name {}", name);
            if name.len() == 0 {
            } else if name == "." {
                continue;
            } else if name == ".." {
                nodes.pop();
                continue;
            } else {
                let child = inode.get_child(name)?;
                nodes.push(child);
            }
        } else {
            // ".." 超过根目录，比如"/dir/../.."
            return Err(FileErr::NotDefine);
        }
    }
    match nodes.last() {
        Some(inode) => {
            log!("path_resolve":"success">"{}", path);
            Ok(inode.clone())
        }
        None => Err(FileErr::NotDefine),
    }
}

pub fn is_absolute_path(path: &str) -> bool {
    match path.chars().next() {
        Some('/') => true,
        _ => false,
    }
}
