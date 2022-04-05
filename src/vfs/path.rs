use super::*;

fn split_path<'c>(path: &'c str) -> (&'c str, Option<&'c str>) {
    // remove trailing slash and split into 2 components - top-most parent and rest
    let mut path_split = path.trim_matches('/').splitn(2, '/');
    let comp = path_split.next().unwrap(); // SAFE: splitn always returns at least one element
    let rest_opt = path_split.next();
    (comp, rest_opt)
}