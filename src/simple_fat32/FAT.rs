

pub struct FAT {
    // fat表1所在块
    fat1: u32,
    // fat表2所在块
    fat2: u32,
    // fat表大小，单位块
    _size: u32,
}