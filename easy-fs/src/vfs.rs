
///
/// 索引节点
/// 服务于文件相关的系统调用
/// 对于文件系统的使用者而言，他们并不关心磁盘布局是如何实现的
/// 只希望看到目录树结构中的逻辑上的文件和目录
/// 为此设计 Inode 暴露给使用者，对文件及目录直接操作
/// 

pub struct Inode {
    block_id: usize,
    block_offset: usize,
    fs: Arc<Mutex<EasyFileSystem>>,
    block_device: Arc<dyn BlockDevice>,
}

impl Indoe {
    fn read_disk_inode<V>(&self, f: impl FnOnce(&DiskInode) -> V) -> V {
        get_block_cache(
            self.block_id,
            Arc::clone(&self.block_device)
        ).lock().read(self.block_offset, f)
    }

    fn modify_disk_inode<V>(&self, f: impl FnOnce(&mut DiskInode) -> V) -> V {
        get_block_cache(
            self.block_id,
            Arc::clone(&self.block_device),
        ).lock().modify(self.block_offset, f)
    }

    pub fn new(
        block_id: u32,
        block_offset: usize,
        fs: Arc<Mutex<EasyFileSystem>>,
        block_device: Arc<dyn BlockDevice>,
    ) -> Self {
        Self {
            block_id: block_id as usize,
            block_offset,
            fs,
            block_device,
        }
    }

    pub fn find(&self, name: &str) -> Option<Arc<Inode>> {
        let fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| {
            self.find_inode_id(name, disk_inode)
            .map(|indoe_id| {
                let (block_id, block_offset) = fs.get_disk_inode_pos(indoe_id);
                Arc::new(Self::new(
                    block_id,
                    block_offset,
                    self.fs.clone(),
                    self.block_device.clone(),
                ))
            })
        })
    }

    fn find_inode_id(
        &self,
        name: &str,
        disk_indoe: &DiskIndoe,
    ) -> Option<u32> {
        // assert it is a directory
        assert!(disk_indoe.is_dir());
        let file_count = (disk_indoe.size as usize) / DIRENT_SIZE;
        let mut dirent = DirEntry::empty();
        for i in 0..file_count {
            assert_eq!(
                disk_inode.read_at(
                    DIRENT_SIZE * i,
                    dirent.as_bytes_mut(),
                    &self.block_device,
                ),
                DIRENT_SIZE,
            );
            if dirent.name() == name {
                return Some(dirent.inode_number() as u32);
            }
        }
        None
    }
    
}