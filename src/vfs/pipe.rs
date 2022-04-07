use alloc::sync::Arc;
use spin::Mutex;
use super::*;

#[derive(Default)]
struct PipeInode {
    inner: Mutex<PipeInner>
}

const PIPE_INODE_SIZE: usize = 512;
struct PipeInner {
    // 记录上一次未完成的读操作已读数据
    last_read: usize,
    // 记录上一次未完成的写操作已写数据
    last_write: usize,
    nread: usize,
    nwrite: usize,
    data: [u8; PIPE_INODE_SIZE]
}

impl Default for PipeInner {
    fn default() -> Self {
        Self {
            last_read: 0,
            last_write: 0,
            nread: 0,
            nwrite: 0,
            data: [0; PIPE_INODE_SIZE]
        }
    }
}

impl _Inode for PipeInode {
    fn read_offset(&self, _: usize, buf: &mut [u8]) -> Result<usize, FileErr> {
        // 管道读时忽略offset参数
        let mut inner = self.inner.lock();
        let len = buf.len();
        let mut i = inner.last_read;
        while i < len {
            if inner.nread == inner.nwrite {
                inner.last_read = i;
                // Todo: 当另一端关闭时不再等待
                log!("pipe":"read">"remain (len: {}, i: {}, nread: {})", len, i, inner.nread);
                return Err(FileErr::PipeReadWait)
            }
            buf[i] = inner.data[inner.nread % PIPE_INODE_SIZE];
            inner.nread += 1;
            i += 1;
        }
        inner.last_read = 0;
        return Ok(i)
    }

    fn write_offset(&self, _: usize, buf: &[u8]) -> Result<usize, FileErr> {
        // 管道写时忽略offset参数
        let mut inner = self.inner.lock();
        let len = buf.len();
        let mut i = inner.last_write;
        while i < len {
            if inner.nwrite == inner.nread + PIPE_INODE_SIZE {
                inner.last_write = i;
                // Todo: 当另一端关闭时不再等待
                log!("pipe":"write">"remain (len: {}, i: {}, nwrite: {})", len, i, inner.nwrite);
                return Err(FileErr::PipeWriteWait)
            }
            let off = inner.nwrite % PIPE_INODE_SIZE;
            inner.data[off] = buf[i];
            inner.nwrite += 1;
            i += 1;
        }
        inner.last_write = 0;
        return Ok(i)
    }
}

pub fn make_pipe() -> Result<(File, File), FileErr> {
    let pipe = Arc::new(PipeInode::default());
    let reader = File::open(pipe.clone(), OpenFlags::RDONLY)?;
    let writer = File::open(pipe, OpenFlags::WRONLY)?;
    Ok((reader, writer))
}