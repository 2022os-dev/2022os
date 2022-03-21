//磁盘块大小，单位字节
const BLOCK_SIZE: u32 = 512;


pub struct DevInodeInfo {
    //对应的位于磁盘的索引节点位于的块号
    block_id: usize,
    //块内偏移，只有0，128，256，384
    block_offset: usize,
    //超级块字段，需要此块来分配inode和datablock
    sb: Arc<Mutex<DevSuperBlock>>,
}

impl DevInodeInfo {
    /// We should not acquire efs lock here.
    pub fn new(
        block_id: u32,
        block_offset: usize,
        sb: Arc<Mutex<DevSuperBlock>>
    ) -> Self {
    }

    fn read_disk_inode<V>(&self, f: impl FnOnce(&DiskInode) -> V) -> V {
        get_block_cache(self.block_id, Arc::clone(&self.block_device))
            .lock()
            .read(self.block_offset, f)
    }

    fn modify_disk_inode<V>(&self, f: impl FnOnce(&mut DiskInode) -> V) -> V {
        get_block_cache(self.block_id, Arc::clone(&self.block_device))
            .lock()
            .modify(self.block_offset, f)
    }

    fn find_inode_id(&self, name: &str, disk_inode: &DiskInode) -> Option<u32> {
        assert!(disk_inode.is_dir());
        let file_count = (disk_inode.size as usize) / DIRENT_SZ;
        let mut dirent = DirEntry::empty();
        for i in 0..file_count {
            assert_eq!(
                disk_inode.read_at(DIRENT_SZ * i, dirent.as_bytes_mut(), &self.block_device,),
                DIRENT_SZ,
            );
            if dirent.name() == name {
                return Some(dirent.inode_number() as u32);
            }
        }
        None
    }

    pub fn find(&self, name: &str) -> Option<Arc<Inode>> {
        let fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| {
            self.find_inode_id(name, disk_inode).map(|inode_id| {
                let (block_id, block_offset) = fs.get_disk_inode_pos(inode_id);
                Arc::new(Self::new(
                    block_id,
                    block_offset,
                    self.fs.clone(),
                    self.block_device.clone(),
                ))
            })
        })
    }

    fn increase_size(
        &self,
        new_size: u32,
        disk_inode: &mut DiskInode,
        fs: &mut MutexGuard<EasyFileSystem>,
    ) {
        if new_size < disk_inode.size {
            return;
        }
        let blocks_needed = disk_inode.blocks_num_needed(new_size);
        let mut v: Vec<u32> = Vec::new();
        for _ in 0..blocks_needed {
            v.push(fs.alloc_data());
        }
        disk_inode.increase_size(new_size, v, &self.block_device);
    }
    
    //修改参数，加入位于哪个目录创建
    pub fn createInode(&self, name: &str) -> Option<Arc<Inode>> {
        let mut fs = self.fs.lock();
        let op = |root_inode: &DiskInode| {
            // assert it is a directory
            assert!(root_inode.is_dir());
            // has the file been created?
            self.find_inode_id(name, root_inode)
        };
        if self.read_disk_inode(op).is_some() {
            return None;
        }
        // create a new file
        // alloc a inode with an indirect block
        let new_inode_id = fs.alloc_inode();
        // initialize inode
        let (new_inode_block_id, new_inode_block_offset) = fs.get_disk_inode_pos(new_inode_id);
        get_block_cache(new_inode_block_id as usize, Arc::clone(&self.block_device))
            .lock()
            .modify(new_inode_block_offset, |new_inode: &mut DiskInode| {
                new_inode.initialize(DiskInodeType::File);
            });
        self.modify_disk_inode(|root_inode| {
            // append file in the dirent
            let file_count = (root_inode.size as usize) / DIRENT_SZ;
            let new_size = (file_count + 1) * DIRENT_SZ;
            // increase size
            self.increase_size(new_size as u32, root_inode, &mut fs);
            // write dirent
            let dirent = DirEntry::new(name, new_inode_id);
            root_inode.write_at(
                file_count * DIRENT_SZ,
                dirent.as_bytes(),
                &self.block_device,
            );
        });

        let (block_id, block_offset) = fs.get_disk_inode_pos(new_inode_id);
        block_cache_sync_all();
        // return inode
        Some(Arc::new(Self::new(
            block_id,
            block_offset,
            self.fs.clone(),
            self.block_device.clone(),
        )))
        // release efs lock automatically by compiler
    }

    pub fn ls(&self) -> Vec<String> {
        
    }

    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        
    }

    pub fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        
    }

    pub fn clear(&self) {
        
    }
}


impl InodeOp for DevInodeInfo {
    // 在dir对应的目录Inode下创建一个Inode
    fn create(&self, dir: &mut Inode, name: String) -> Result<*mut Inode, InodeOpErr> {
        createInode();
    }

    // 在dir对应的目录Inode下搜索一个名为name的Inode
    fn lookup(&self, dir: &mut Inode, name: String) -> Result<*mut Inode, InodeOpErr> {
        find();

    }

    // 在dir对应的目录Inode下创建一个目录Inode，可以使用create来实现
    fn mkdir(&self, dir: *mut Inode, name: String, flag: InodeFlag) -> Result<(), InodeOpErr> {
        createInode();
    }

    // 在dir对应的目录Inode下删除一个名为name的目录，删除的策略由InodeFlag来指定
    fn rmdir(&self, dir: *mut Inode, name: String) -> Result<(),InodeOpErr> {
        let sb = self.sb.lock();
        sb.dealloc_inode();
    }

    // 将old_inode目录Inode下的old_name对应的dentry移动到new_inode对应的new_namedentry中
    fn rename(&self, old_dir: *mut Inode, old_name: String, new_dir: *mut Inode, new_name: String) -> Result<(), InodeOpErr> {
        let sb = self.sb.lock();
        sb.dealloc_inode();
        createInode();
    }

    // 改变文件大小，调用前先将Inode的i_size修改，将文件大小修改为i_size
    fn truncate(&self, inode: *mut Inode) -> Result<(), ()> {
        increase_size();
    }

    // 获取inode对应的文件偏移为offset字节的内容所在的块号
    fn bmap(&self, inode: *mut Inode, offset: usize) -> Result<Ino, InodeOpErr> {
        let block_id = (offset + BLOCK_SIZE - 1) / BLOCK_SIZE;
        //此方法在dev_inode_info中暂未实现
        let block_id = get_block_id();
    }
}