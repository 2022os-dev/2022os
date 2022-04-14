# TODO list

## 其他
- [ ] 检查系统调用接口中usize和isze的转换，可能会溢出导致内核panic
- [ ] 检查系统调用接口中数据类型不匹配的情况，比如sizeof(int) == 4，但是使用了usize
- [ ] shell
  - [x] 能运行用户态程序
  - [ ] 内置命令
- [ ] uname获取系统信息

## 调度
- [ ] 使用无锁队列调度，提高并发
- [ ] 就绪队列无任务时hart休眠，有任务时唤醒
- [ ] 系统调用
  - [ ] clone
    - [x] fork时复制文件描述符
    - [ ] 处理clone flags
  - [x] exit
  - [ ] wait4
    - [x] 阻塞等待子进程退出
    - [ ] 处理wait4选项
    - [x] 正确写入wstatus
  - [x] getpid
  - [x] getppid
  - [x] yield

## 内存管理
- [ ] 检查用户传入的虚拟地址是否有效，若地址无效会导致内核错误
- [ ] 将内核的堆内存分配统一为从kalloc接口分配
- [ ] Copy on write
- [ ] 系统调用
  - [ ] mmap
  - [ ] munmap
  - [x] brk
  - [x] sbrk

## 信号处理
- [ ] 信号队列使用Atomic，提高并发
- [ ] 嵌套信号处理
- [ ] 系统产生更多信号
- [ ] 处理sigaction选项
- [ ] 信号处理函数返回时跳转调用sigreturn
- [ ] 系统调用
  - [ ] sigaction
  - [x] kill
  - [x] sigreturn
  - [ ] sigmask

## 文件系统
- [ ] 稳定的vfs接口
- [ ] 文件系统挂载管理
- [ ] 异步处理IO系统调用
- [ ] fat32文件系统集成
- [ ] 处理文件mode
- [ ] 设置出错码errno
- [ ] 实现dentry缓存
- [ ] 文件系统调用
  - [ ] execve
    - [x] 基本的execve调用
    - [ ] 处理O_CLOEXEC
  - [ ] getcwd
    - [ ] 当buf为0表示由内核分配内存(malloc)
    - [x] pcb结构保存cwd
    - [x] 返回cwd
  - [ ] pipe2
    - [x] 管道Inode结构
    - [x] 没有数据时阻塞等待另一方读/写
    - [x] 多进程测试
    - [ ] 解决单进程写入或读出大于管道缓冲大小的内容会一直阻塞的问题
    - [x] 当另一端关闭时候不再等待
  - [x] dup 
  - [ ] dup3
    - [x] 复制fd
    - [ ] 处理flags
  - [ ] chdir
    - [x] 判断目标路径是否存在
    - [x] 更改cwd
    - [ ] 去掉目录中的"."和".."
  - [x] openat
    - [x] 解析绝对路径
    - [x] 解析相对路径
    - [x] 创建文件
    - [x] 在指定的文件描述符打开文件
    - [x] 路径解析".."、"."
  - [x] close
  - [ ] getdents
    - [x] 返回目录项
    - [x] 判断fd的open flags能否读取目录
    - [ ] 构造".."和"."目录项
  - [x] read
  - [x] write
  - [x] linkat 
  - [ ] unlinkat
    - [x] 删除文件
    - [ ] 处理REMOVEDIR flags
  - [x] mkdirat 
    - [x] 创建绝对路径文件夹
    - [x] 创建相对路径文件夹
    - [x] 处理".."、"."
  - [ ] umount2
  - [ ] mount
  - [ ] fstat 

## SD卡驱动
- [ ] SPI协议驱动