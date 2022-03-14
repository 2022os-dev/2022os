# 文件IO文档

## VFS接口文档

- VFS接口主要主要包含三个对象SupberBlock, Inode, 和File。SuperBlock表示文件系统的元数据，Inode表示磁盘中文件的元数据，这里“磁盘中文件”即包含了普通文件，也包含了目录、符号链接等。还有File对象表示进程文件。其中SuperBlock和Inode对象是文件系统主要关注的对象。文件系统主要实现SuperBlockOp和InodeOp。参考linux 1.0源码中fs/ext、fs/ext2的实现。下面接口设计参考了linux 1.0中的 include/linux/fs.h、《深入理解Linux内核》、xv6等资料。
- File结构暂时未定义，文件系统应该不会使用到File结构。
- 目前大部分引用暂时使用祼指针，因为Rust严格的所有权检查让会让文件系统的实现变得很困难，在后续实现中若有机会应该慢慢将裸指针使用引用代替。
- 目前对多核的考虑不全面，很多地方使用了线程不安全的结构。

	- SuperBlock 
		- 表示文件系统元数据
		- 成员
			- s_dev: 设备驱动号，目前因为没支持设备管理，使用0表示磁盘设备。
			- s_blocksize: 文件系统块大小，以字节为单位。
			- s_dirt: 文件系统修改标志，表明文件系统需要与磁盘同步。
			- s_maxbytes: 最大支持文件的长度。
			- s_op: 由文件系统实现的接口。
			- s_inodes: 文件系统相关的所有Inode对象
			- s_fs_info: 文件系统自定义结构。
			- s_root: 文件系统根目录对应的Inode。
		- s_op 操作，是文件系统的主要接口，文件系统需要定义一个结构实现s_op。
			- fsinit: 文件系统初始化。
			- alloc_inode: 分配一个与文件系统关联的Inode对象，
			- delete_inode: 在内存与磁盘同时删除Inode节点，需要检查引用数是否归0 。
			- read_inode: 从磁盘读取inode数据，必须保证inode的i_ino被正确填写，通过i_ino读取inode
			- write_inode: 将Inode结构写回磁盘。
			- put_inode: 将Inode引用数减1 。
			- put_super: 文件系统被取消挂载，可以暂时不做实现，因为目前只有根文件系统，不会取消。
			- write_super: 与磁盘同步SuperBlock。
	- Inode
		- Inode是内存中保存磁盘文件元数据的结构，磁盘文件与目录都对应一个Inode对象，通过i_type区分不同的Inode类型。
		- 成员
			- i_ino: 磁盘块号，每个Inode与一个磁盘块绑定，指定i_ino后通过SuperBlockOp的read_inode将Inode结构填写完成。
			- i_type: Inode类型，目前支持File和Directory类型。
			- i_count: 引用计数，用于磁缓冲磁盘IO，当引用计数为0，释放内存中对磁盘块内容的缓存。使用了原子类型，不需要锁住Inode。
			- i_nlink: 文件的链接数（硬链接）。
			- i_dirt: 脏标志，Inode应该与磁盘内容同步。
			- i_uid: 目前不支持多用户，置0即可。
			- i_gid: 目前不支持组，置0即可。
			- i_size: 文件大小，单位为字节。
			- i_atime: 最近访问中时间，暂时可不管。
			- i_mtime: 最近写文件时间，暂时可不管。
			- i_ctime: 最近修改时间，暂时可不管。
			- i_blksize: 块的字节数，单位为字节。
			- i_blocks: 文件占用的块数，不足一块时计作一块。
			- i_op: Inode操作
			- i_lock: Inode对象锁。
			- i_sb: Inode对象对应的SuperBlocks。
		- i_op 操作
			- create: 在dir对应的目录Inode下创建一个Inode。
			- lookup: 在dir对应的目录Inode下搜索一个名为name的Inode。
			- link: 在dir对应的目录Inode下创建硬链接，硬链接的名称为name，指向old_inode对应的Inode。实际的实现可能为在dir下创建一个dentry，dentry的名称为name，dentry指向的ino为old_inode->i_ino。
			- symlink: 在dir对应的目录Inode下创建符号链接，符号链接的名称是name，符号链接文件指向一个文件路径path，该路径可能为相对路径。
			- mkdir: 在dir对应的目录Inode下创建一个目录Inode，可以使用create来实现。
			- rmdir: 在dir对应的目录Inode下删除一个名为name的目录，删除的策略由InodeFlag来指定。
			- mknod: 先不用实现。
			- rename: 将old_inode目录Inode下的old_name对应的dentry移动到new_inode对应的new_namedentry中。
			- readlink: 将link对应的符号链接Inode指向的符号链接绝对地址返回。
			- follow_link: 寻找符号链接inode指向的Inode，如果符号链接内的文件路径为相对路径，则从dir开始解析的，dir为link的父目录。
			- truncate: 改变文件大小，调用前先将Inode的i_size修改，将文件大小修改为i_size。
			- bmap: 获取inode对应的文件偏移为offset字节的内容所在的块号。
## 文件名称解析
- 该部分由内核完成，暂未实现。
### 文件目录项缓存
- 内核缓存文件路径对应的Inode内存结构，避免对目录项的频繁读写。
## 块缓存
- 文件系统通过块缓存提供的接口对文件进行读写。块缓存目前未实现，只是提供了一个供文件系统读取磁盘的接口。代码src/vfs/dcache.rs
	- 函数
		- get_block(dev: usize, ino: Ino) -> Arc\<Buffer>
			- 获取设备特定的块号，返回一个Buffer结构的指针。Buffer结构包含块的数据。由于使用了Arc共享指针，可以不显式地更新Buffer的引用数，当Buffer的引用数归0后，调用Buffer的Drop函数进行写回。
	- Buffer结构
		- Buffer结构内包含一个字节数组保存磁盘数据，数组大小由BLOCK_BUFFER_SIZE指定，可以修改为合适的数据。
		- 通过get_block获取Buffer结构后可以按如下方式使用。
		```
		{
			let buf = get_block(0, 0);
			// 获取 buf 数据 的锁
			let data = buf.data();
			// 读
			let byte = data[10];
			// 写
			data[11] = 128;
			// buf的 数据 释放锁
		}

		```