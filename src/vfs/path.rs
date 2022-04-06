use super::*;

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
    let (name, rest) = split_path(path);
    if name.len() == 0 {
        return Ok(from.clone())
    }
    // 递归调用
    from.get_child(name).and_then(|inode| {
        if let Some(rest) = rest {
            parse_path(&inode, rest)
        } else {
            Ok(inode)
        }
    })

}

pub fn is_absolute_path(path: &str) -> bool {
    match path.chars().next() {
        Some('/') => true,
        _ => false
    }
}