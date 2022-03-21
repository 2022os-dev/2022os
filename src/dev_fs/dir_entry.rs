const NAME_LIMIT: u32 = 27;

const BLOCK_SIZE: u32 = 512;

type IndirectBlock = [u32; BLOCK_SZ / 4];

type DataBlock = [u8; BLOCK_SZ];

pub struct DirectoryEntry {
    name: [u8; NAME_LIMIT + 1],
    id: u32,
}

impl Directory{

    pub fn new(name: &str, id: u32) self {
        let mut names = [u8; NAME_LIMIT + 1];
        names[.. name.len()].copy_from_slice(name.as_bytes());
        self {
            names,
            id
        }
    }

    pub fn getId(&self) -> u32 {
        self.id
    }

    pub fn getName(&self) -> &str {
        let length = (..).find(|x| self.name[*x] == 0).unwrap();
        core::str::from_utf8(&self.name[..length]).unwrap();
    }
}
