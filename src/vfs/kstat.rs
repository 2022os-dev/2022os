pub const S_IFMT   :u32 =  0170000;   //bit mask for the file type bit field
pub const S_IFSOCK :u32 =  0140000;   //socket
pub const S_IFLNK  :u32 =  0120000;   //symbolic link
pub const S_IFREG  :u32 =  0100000;   //regular file
pub const S_IFBLK  :u32 =  0060000;   //block device
pub const S_IFDIR  :u32 =  0040000;   //directory
pub const S_IFCHR  :u32 =  0020000;   //character device
pub const S_IFIFO  :u32 =  0010000;   //FIFO

pub const S_ISUID  :u32 =   04000;   //set-user-ID bit (see execve(2))
pub const S_ISGID  :u32 =   02000;   //set-group-ID bit (see below)
pub const S_ISVTX  :u32 =   01000;   //sticky bit (see below)
pub const S_IRWXU  :u32 =   00700;   //owner has read, write, and execute

pub const S_IRUSR  :u32 =   00400;   //owner has read permission
pub const S_IWUSR  :u32 =   00200;   //owner has write permission
pub const S_IXUSR  :u32 =   00100;   //owner has execute permission
pub const S_IRWXG  :u32 =   00070;   //group has read, write, and execute

pub const S_IRGRP  :u32 =   00040;   //group has read permission
pub const S_IWGRP  :u32 =   00020;   //group has write permission
pub const S_IXGRP  :u32 =   00010;   //group has execute permission
pub const S_IRWXO  :u32 =   00007;   //others (not in group) have read, write,

pub const S_IROTH  :u32 =   00004;   //others have read permission
pub const S_IWOTH  :u32 =   00002;   //others have write permission
pub const S_IXOTH  :u32 =   00001;   //others have execute permission


#[repr(C)]
pub struct Kstat {
	st_dev: u32, /* ID of device containing file */
	st_ino: u32, /* Inode number */
	st_mode: u32, /* File type and mode */
	st_nlink: u32, /* Number of hard links */
	st_uid: u32, /* User ID of owner */
	st_gid: u32, /* Group ID of owner */
	st_rdev: u32, /* Device ID (if special file) */
	long_pad: u32, 
	st_size: u32, /* Total size, in bytes */
	st_blksize: u32, /* Block size for filesystem I/O */
	_pad2: u32,
	st_blocks: u32, /* Number of 512B blocks allocated */
	st_atime_sec : i64, 
    st_atime_nsec: i64,  
    st_mtime_sec : i64,  
    st_mtime_nsec: i64,   
    st_ctime_sec : i64,  
    st_ctime_nsec: i64,  
}

impl Kstat {
    pub fn empty() -> Self {
        Self {
            st_dev: 0,
	        st_ino: 0,
	        st_mode: 0,
	        st_nlink: 0,
	        st_uid: 0,
	        st_gid: 0,
	        st_rdev: 0,
	        long_pad: 0,
	        st_size: 0,
	        st_blksize: 0,
	        _pad2: 0,
	        st_blocks: 0,
	        st_atime_sec : 0, 
            st_atime_nsec: 0, 
            st_mtime_sec : 0, 
            st_mtime_nsec: 0,   
            st_ctime_sec : 0,  
            st_ctime_nsec: 0,  
        }
    }

    pub fn create(
        &mut self, 
        st_atime_sec: i64, 
        st_mtime_sec: i64, 
        st_ctime_sec: i64, 
        st_size: u32, 
        st_dev: u32, 
        st_ino: u32, 
        st_mode: u32, 
        st_nlink: u32) {
            *self = Self {
                st_dev: st_dev,
	            st_ino: st_ino,
	            st_mode: st_mode,
	            st_nlink: st_nlink,
	            st_uid: 0,
	            st_gid: 0,
	            st_rdev: 0,
	            long_pad: 0,
	            st_size: st_size,
	            st_blksize: 512,
	            _pad2: 0,
	            st_blocks: (st_size + self.st_blksize - 1) / self.st_blksize,
	            st_atime_sec : st_atime_sec, 
                st_atime_nsec: 0, 
                st_mtime_sec : st_mtime_sec, 
                st_mtime_nsec: 0,   
                st_ctime_sec : st_ctime_sec,  
                st_ctime_nsec: 0,  
        }
    }

	//is it a regular file?
	pub fn is_reg (&self) -> bool { 
		self.st_mode & S_IFMT == S_IFREG
	}

	//directory?
	pub fn is_dir (&self) -> bool { 
		self.st_mode & S_IFMT == S_IFDIR
	}
                  

	//character device?
	pub fn is_chr (&self) -> bool { 
		self.st_mode & S_IFMT == S_IFCHR
	}

	//block device?
	pub fn is_blk (&self) -> bool { 
		self.st_mode & S_IFMT == S_IFBLK
	}

	//FIFO (named pipe)?
	pub fn is_ifo (&self) -> bool { 
		self.st_mode & S_IFMT == S_IFIFO
	}

	//symbolic link?  (Not in POSIX.1-1996.)
	pub fn is_lnk (&self) -> bool { 
		self.st_mode & S_IFMT == S_IFLNK
	}

	//socket?
	pub fn is_sock (&self) -> bool { 
		self.st_mode & S_IFMT == S_IFSOCK
	}
                  

           
}