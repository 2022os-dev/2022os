use alloc::sync::Arc;
use spin::Mutex;
use super::*;

#[derive(Default)]
struct PipeInode {
    inner: Mutex<PipeInner>
}

const PIPE_INODE_SIZE: usize = 512;
struct PipeInner {
    read_ready: bool,
    write_ready: bool,
    // 记录有几个文件读该Inode
    reader: usize,
    // 记录有几个文件写该Inode
    writer: usize,
    // 记录上一次未完成的读操作已读数据
    last_read: usize,
    // 记录上一次未完成的写操作已写数据
    last_write: usize,
    // 记录总共已读字节数
    nread: usize,
    // 记录总共已写字节数
    nwrite: usize,
    data: [u8; PIPE_INODE_SIZE]
}

impl Default for PipeInner {
    fn default() -> Self {
        Self {
            read_ready: true,
            write_ready: true,
            reader: 0,
            writer: 0,
            last_read: 0,
            last_write: 0,
            nread: 0,
            nwrite: 0,
            data: [0; PIPE_INODE_SIZE]
        }
    }
}

impl _Inode for PipeInode {

    fn len(&self) -> usize {
        usize::MAX
    }

    fn read_offset(&self, _: usize, buf: &mut [u8]) -> Result<usize, FileErr> {
        // 管道读时忽略offset参数
        let mut inner = self.inner.lock();
        if inner.writer == 0 {
            // 另一端已经关闭，不会有数据写入，返回0
            return Ok(buf.len())
        }
        let len = buf.len();
        let mut i = inner.last_read;
        while i < len {
            if inner.nread == inner.nwrite {
                // 需要等待另一端写入才能读出，记录这次未完成的读位置，下次读时继续
                inner.last_read = i;
                log!("pipe":"read">"remain (len: {}, i: {}, nread: {})", len, i, inner.nread);
                inner.read_ready = false;
                return Err(FileErr::PipeReadWait)
            }
            buf[i] = inner.data[inner.nread % PIPE_INODE_SIZE];
            inner.nread += 1;
            i += 1;
        }
        inner.write_ready = true;
        inner.last_read = 0;
        return Ok(i)
    }

    fn write_offset(&self, _: usize, buf: &[u8]) -> Result<usize, FileErr> {
        // 管道写时忽略offset参数
        let mut inner = self.inner.lock();
        if inner.reader == 0 {
            // 另一端已经关闭，不会将数据读出，返回0
            return Ok(buf.len())
        }
        let len = buf.len();
        let mut i = inner.last_write;
        while i < len {
            if inner.nwrite == inner.nread + PIPE_INODE_SIZE {
                // 需要等待另一端读出才能写入，记录这次未完成的写位置，下次写时继续
                inner.last_write = i;
                log!("pipe":"write">"remain (len: {}, i: {}, nwrite: {})", len, i, inner.nwrite);
                inner.write_ready = false;
                return Err(FileErr::PipeWriteWait)
            }
            let off = inner.nwrite % PIPE_INODE_SIZE;
            inner.data[off] = buf[i];
            inner.nwrite += 1;
            i += 1;
        }
        inner.read_ready = true;
        inner.last_write = 0;
        return Ok(i)
    }

    fn file_open(&self, flags: OpenFlags) {
        let mut inner = self.inner.lock();
        if flags.readable() {
            inner.reader += 1;
        }
        if flags.writable() {
            inner.writer += 1;
        }
    }

    fn file_close(&self, file: &File) {
        let mut inner = self.inner.lock();
        if file.flags().readable() {
            inner.reader -= 1;
        }
        if file.flags().writable() {
            inner.writer -= 1;
        }
    }
}

pub fn make_pipe() -> Result<(Fd, Fd), FileErr> {
    let pipe = Arc::new(PipeInode::default());
    let reader = File::open(pipe.clone(), OpenFlags::RDONLY)?;
    let writer = File::open(pipe, OpenFlags::WRONLY)?;
    Ok((reader, writer))
}