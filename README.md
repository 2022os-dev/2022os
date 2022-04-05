# TODO list

## 其他
- [ ] 检查系统调用接口中usize和isze的转换，可能会溢出导致内核panic
- [ ] shell
- [ ] uname获取系统信息

## 调度
- [ ] 使用无锁队列调度，提高并发
- [ ] 就绪队列无任务时hart休眠，有任务时唤醒
- [ ] 系统调用
  - [ ] clone
    - [ ] fork时复制文件描述符
  - [x] exit
  - [x] wait4
  - [x] getpid
  - [x] getppid
  - [x] yield

## 内存管理
- [ ] 检查用户传入的虚拟地址是否有效，若地址无效会导致内核错误
- [ ] 将内核的堆内存分配统一为从kalloc接口分配
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
- [ ] 文件系统调用
  - [ ] execve
  - [x] getcwd
  - [ ] pipe2
  - [ ] dup 
  - [ ] dup3
  - [ ] chdir
  - [ ] openat
  - [ ] close
  - [ ] getdents
  - [x] read
  - [x] write
  - [ ] linkat 
  - [ ] unlinkat
  - [ ] mkdirat 
  - [ ] umount2
  - [ ] mount
  - [ ] fstat 

## SD卡驱动
- [ ] SPI协议驱动