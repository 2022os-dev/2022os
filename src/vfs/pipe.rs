use super::*;
use alloc::sync::Arc;
use spin::Mutex;

#[derive(Default)]
struct PipeInode {
    inner: Mutex<PipeInner>,
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
    data: [u8; PIPE_INODE_SIZE],
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
            data: [0; PIPE_INODE_SIZE],
        }
    }
}

impl _Inode for PipeInode {
    fn len(&self) -> usize {
        usize::MAX
    }

    fn read_offset(&self, _: usize, buf: &mut [u8]) -> Result<usize, FileErr> {
        // 管道读时忽略offset参数
        log!("pipe":"read_offset">"len ({})", buf.len());
        let mut inner = self.inner.lock();
        while inner.last_read < buf.len() {
            if inner.nread == inner.nwrite {
                // 读取少于buf.len()字节，可能需要等待另一端写入
                if inner.writer == 0 {
                    // 另一端已经关闭，不再等待
                    let read_size = inner.last_read;
                    inner.last_read = 0;
                    return Ok(read_size);
                }
                inner.write_ready = true;
                inner.read_ready = false;
                // 返回PipeReadWait，使进程陷入阻塞，见src/trap/syscall/file.rs:sys_read
                return Err(FileErr::PipeReadWait);
            }
            buf[inner.last_read] = inner.data[inner.nread % PIPE_INODE_SIZE];
            inner.nread += 1;
            inner.last_read += 1;
        }
        inner.write_ready = true;
        // 成功读取到buf.len()字节，恢复last_read
        inner.last_read = 0;
        return Ok(buf.len());
    }

    fn write_offset(&self, _: usize, buf: &[u8]) -> Result<usize, FileErr> {
        // 管道写时忽略offset参数
        log!("pipe":"write_offset">"len ({})", buf.len());
        let mut inner = self.inner.lock();
        while inner.last_write < buf.len() {
            if inner.nwrite == inner.nread + PIPE_INODE_SIZE {
                // 写入少于buf.len()字节，可能需要等待另一端读取
                if inner.reader == 0 {
                    // 另一端已经关闭，不再等待
                    let write_size = inner.last_write;
                    inner.last_write = 0;
                    return Ok(write_size);
                }
                inner.read_ready = true;
                inner.write_ready = false;
                // 返回PipeWriteWait，使进程陷入阻塞，见src/trap/syscall/file.rs:sys_write
                return Err(FileErr::PipeWriteWait);
            }
            let off = inner.nwrite % PIPE_INODE_SIZE;
            inner.data[off] = buf[inner.last_write];
            inner.nwrite += 1;
            inner.last_write += 1;
        }
        inner.read_ready = true;
        // 成功写入到buf.len()字节，恢复last_write
        inner.last_write = 0;
        return Ok(buf.len());
    }

    fn file_open(&self, flags: OpenFlags) {
        let mut inner = self.inner.lock();
        if flags.readable() {
            inner.reader += 1;
        }
        if flags.writable() {
            inner.writer += 1;
        }
        log!("pipe":"file_open">"remain reader({}), remain writer({})", inner.reader, inner.writer);
    }

    fn file_close(&self, file: &File) {
        let mut inner = self.inner.lock();
        if file.flags().readable() {
            inner.reader -= 1;
        }
        if file.flags().writable() {
            inner.writer -= 1;
        }
        log!("pipe":"file_close">"remain reader({}), remain writer({})", inner.reader, inner.writer);
    }

    fn read_ready(&self) -> bool {
        let inner = self.inner.lock();
        // 如果没有写入者时可以read
        if inner.writer == 0 {
            true
        } else {
            inner.read_ready
        }
    }

    fn write_ready(&self) -> bool {
        let inner = self.inner.lock();
        // 如果没有读出者时可以write
        if inner.reader == 0 {
            true
        } else {
            inner.write_ready
        }
    }
}

pub fn make_pipe() -> Result<(Fd, Fd), FileErr> {
    let pipe = Arc::new(PipeInode::default());
    let reader = File::open(pipe.clone(), OpenFlags::RDONLY)?;
    let writer = File::open(pipe, OpenFlags::WRONLY)?;
    Ok((reader, writer))
}
