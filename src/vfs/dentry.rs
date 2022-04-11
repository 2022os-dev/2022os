use crate::config::PATH_LIMITS;

#[repr(C)]
pub struct LinuxDirent {
    pub d_ino: usize,
    pub d_off: isize,
    pub d_reclen: u16,
    pub d_type: u8,                // linux manual中d_type应该在d_name后面?
    pub d_name: [u8; PATH_LIMITS], // 使用固定的name长度
}

pub const DT_UNKNOWN: u8 = 0;
pub const DT_DIR: u8 = 4;
pub const DT_REG: u8 = 4; //常规文件

impl LinuxDirent {
    pub fn new() -> Self {
        Self {
            d_ino: 0,
            d_off: 0,
            d_reclen: 0,
            d_type: 0,
            d_name: [0; PATH_LIMITS],
        }
    }

    pub fn fill(&mut self, other: &Self) {
        self.d_ino = other.d_ino;
        self.d_off = other.d_off;
        self.d_reclen = other.d_reclen;
        self.d_type = other.d_type;
        self.d_name.copy_from_slice(&other.d_name);
    }
}
