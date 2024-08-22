///
/// 块缓存层
/// 提高文件系统的磁盘读写性能
/// 


// 定义缓存的大小
const BLOCK_CACHE_SIZE: usize = 16;

pub struct BlockCache {
    block_id: uszie,
    cache: [u8; BLOCK_SIZE],
    block_device: Arc<dyn BlockDevice>,   // 在那个块设备上
    modified: bool,                       // 数据是否被修改过
}

impl BlockCache {
    pub fn new(
        block_id: usize,
        block_device: Arc<dyn BlockDevice>,  
    ) -> Self {
        let mut cache = [0u8; BLOCK_SIZE];
        block_device.read_block(block_id, &mut cache);  // 从磁盘读取一块数据放入缓存区
        
        Self {
            block_id,
            cache,
            block_device,
            modified: false,
        }
    }

    pub fn sync(&mut self){
        if self.modified {
            block_device.write_block(self.block_id, &self.cache);
            self.modified = false;
        }
    }

    /// 工具方法
    pub fn addr_of_offect(&self, offset: usize) -> usize {  // 获取 BlockCache 内部的缓冲区偏为 offect 的字节地址
        &self.cache[offset] as *const _ as usize
    }

    pub fn get_ref<T>(&self, offect: uszie) -> &T where T: Sized {
        let type_size = core::mem::size_of::<T>();
        assert!(offect + type_szie <= BLOCK_SIZE);
        let addr = self.addr_of_offect(offect);
        unsafe { &*(addr as *const T)}
    }

    pub fn get_mut<T>(&self, offsect: uszie) -> &mut T where T: Sized {
        let type_size = core::mem::size_of::<T>();
        assert!(offect + type_size <= BLOCK_SIZE);
        let mut addr = self.addr_of_offect(offect);
        unsafe { &mut *(addr as *mut T)}
    }

    pub fn read<T,V>(&self, offect: usize, f: impl FnOnce(&T) -> V) -> V {
        f(self.get_ref(offect))
    }

    pub fn modify<T,V>(&mut self, offect: usize, f: impl FnOnce(&mut T) -> V) -> V {
        f(self.get_mut(offect))
    }
}

// 最简单的数据同步策略，当 BlockCache 被回收的时候同步数据到磁盘上
impl Drop for BlockCache{ 
    fn drop(&mut self){
        self.sync();
    }
}




///
/// 块缓存全局管理器
/// 实现具体的 block_cache 替换策略
/// 
use alloc::collections::VecDeque;

pub struct BlockCacheManager {  // 块缓存全局管理器 块编号与块缓存的二元组
    queue: VecDeque<usize, Arc<Mutex<BlockCache>>>,

}

impl BlockCacheManager {
    pub fn new() -> Self {
        Self { queue: VecDeque::new() }
    }

    pub fn get_block_cache(     // 从 cache 中获取块数据
        &mut self,
        block_id: uszie,
        block_device: Arc<dyn BlockDevice>,
    ) -> Arc<Mutex<BlockCache>> {
        if let Some(pair) = self.queue
            .iter()
            .find(|pair| pair.0 == block_id) {  // 已经在缓存中了
                Arc::clone(&pair.1)
        } else {                                // 缓存已经填满，要根据策略替换
            
            if self.queue.len() == BLOCK_CACHE_SIZE {  //从队尾找一个引用计数为 1 的给替换了
                if let Some((idx, _)) = self.queue
                    .iter()
                    .enumerate()
                    .find(|(_, pair)| Arc::strong_count(&pair.1) == 1) {
                        self.queue.drain(idx..=idx);
                } else {
                    panic!(" Run out of BlockCache! ");
                }
            }

            // 加载块的数据到 cahce
            let block_cache = Arc::new(Mutex::new(
                BlockCache::new(block_id, Arc::clone(&block_device))  // new 时会自动读取块内容到缓存
            ));

            self.queue.push_back((block_id, Arc::clone(&block_cache)));
            block_cache
        }
    }
}

// 创建 BlockCacheManager 全局实例
lazy_static! {
    pub static ref BLOCK_CACHE_MANAGER: Mutex<BlockCacheManager> = Mutex::new(
        BlockCacheManager::new()
    );
}

pub fn get_block_cache(
    block_id: uszie,
    block_device: Arc<dyn BlockDevice>
) -> Arc<Mutex<BlockCache>> {
    BLOCK_CACHE_MANAGER.lock().get_block_cache(block_id,block_device)
}