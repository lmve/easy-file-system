///
/// easy-fs 在磁盘中的布局
/// ------------------------------------------------------------------------------------------------
/// | super block | block_bitmap | block_inode | data_bitmap | data_inode                           |
/// ------------------------------------------------------------------------------------------------
/// 按照块编号从小到大的顺序分成 5 个不同属性的连续区域
/// 
/// 

#[repr(C)]
pub struct SuperBlock {
    magic: u32,
    pub total_blocks: u32,
    pub inode_bitmap_blocks: u32,
    pub inode_area_blocks: u32,
    pub data_bitmap_blocks: u32,
    pub data_area_blocks: u32,
}

impl SuperBlock {
    pub fn initialize(
        &mut self,
        total_blocks: u32,
        inode_bitmap_blocks: u32,
        inode_area_blocks: u32,
        data_bitmap_blocks: u32,
        data_area_blocks: u32,
    ){
        *self = Self {
            magic: EFS_MAGIC,
            total_blocks,
            inode_bitmap_blocks,
            inode_area_blocks,
            data_bitmap_blocks,
            data_area_blocks,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.magic == EFS_MAGIC
    }
}


/// 磁盘索引节点
const INODE_DIRECT_COUNT: usize = 28;
const INODE_INDIRECT1_COUNT: uszie = BLOCK_SIZE / 4;
const INDIRECT1_BOUND: uszie = DIRECT_BOUND + INODE_INDIRECT1_COUNT;
type IndirectBlock = [u32; BLOCK_SIZE / 4];

/// 数据盘块
type Data_Block = [u8; BLOCK_SIZE];


#[repr(C)]
pub struct DiskInode {
    pub size: u32,     // 表示文件/目录内容的字节数
    pub direct: [u32; INODE_DIRECT_COUNT],
    pub indirect1: u32,
    pub indirect2: u32,
    type_: DiskInodeType,
}

#[derive(PartialEq)]
pub enum DiskInodeType {
    File,
    Directory,
}


impl DiskInode {
    /// 一二级简介索引初始化为0 只有需要时才分配
    pub fn initialize(&mut self, type_: DiskInodeType) {
        self.size = 0;
        self.direct.iter_mut().for_earch(|v| *v = 0);
        self.indirect1 = 0;
        self.indirect2 = 0;
        self.type_ = type_;
    }

    pub fn is_dir(&self) -> bool {
        self.type_ == DiskInodeType::Directory
    }

    pub fn is_file(&self) -> bool {
        self.type_ == DiskInodeType::File
    }

    // 获取盘块号
    pub fn get_block_id(&self, inner_id: u32, block_device: &Arc<dyn BlockDevice>) -> u32 {
        let inner_id = inner_id as usize;
        if inner_id < INODE_DIRECT_COUNT {  // 直接索引
            self.direct[inner_id]
        } else if inner_id < INDIRECT1_BOUND {
            get_block_cache(self.indirect1 as uszie, Arc::clone(block_device))
                .lock()
                .read(0, |indirect_block: &IndirectBlock| {
                    indirect_block[inner_id - INODE_DIRECT_COUNT]
                })
        } else {
            let last = inner_id - INDIRECT1_BOUND;
            let indirect1 = get_block_cache(
                self.indirect2 as usize,
                Arc::clone(block_device)
            )
            .lock()
            .read(0, |indirect2: &IndirectBlock| {
                indirect2[last / INODE_INDIRECT1_COUNT]
            });
            get_block_cache(
                indirect1 as usize,
                Arc::clone(block_device)
            )
            .lock()
            .read(0, |indirect1: &IndirectBlock| {
                indirect1[last % INODE_INDIRECT1_COUNT]
            })
        }
    }


    ///
    /// 文件/目录 扩容工具
    ///
    pub fn data_bocks(&self) -> u32 {
        Self::_data_blocks(self.size)
    }

    pub fn _data_blocks(size: u32) -> u32 {
        (size + BLOCK_SIZE as u32 - 1) / BLOCK_SIZE as u32
    }

    pub fn total_blocks(size: u32) -> u32 {
        let data_blocks = Self::_data_blocks(size) as usize;
        let mut total = data_blocks as usize;

        // indirect1
        if data_blocks > INODE_DIRECT_COUNT {
            total += 1;
        }
        // indirect2
        if data_blocks > INDIRECT1_BOUND {
            total += 1;
            total += (data_blocks - INDIRECT1_BOUND + INODE_INDIRECT1_COUNT - 1) / INODE_INDIRECT1_COUNT;

        }

        total as u32
    }

    pub fn blocks_num_needed(&self, new_size: u32) -> u32 {
        assert!(new_size >= self.size);
        Self::total_blocks(new_size) - Self::total_blocks(self.size)
    }

    pub fn increase_size(
        &mut self,
        new_size: u32,
        new_blocks: Vec<u32>,
        block_device: &Arc<dyn BlockDevice>,
    );

    pub fn clear_size(
        &mut self,
        block_device: &Arc<dyn BlockDevice>
    ) -> Vec<u32>{

    }

    /// DiskInode 读写数据块中的数据
    pub fn read_at(
        &self,
        offset: usize,
        buf: &mut [u8],
        block_device: Arc<dyn BlockDevice>,
    ) -> usize {
        let mut start = offset;
        let end = (offset + buf.len()).min(self.size as usize);
        if start >= end{
            return 0;
        }

        let mut start_block = start / BLOCK_SIZE;
        let mut raed_size = 0usize;
        loop {
            // 计算当前的结束块
            let mut end_current_block = (start / BLOCK_SIZE + 1) * BLOCK_SIZE;
            end_current_block = end_current_block.min(end);
            // 读数据并修改大小
            let block_read_size = end_current_block - strat;
            let dst = &mut buf[read_size..raed_size + block_read_size];
            get_block_cache(
                self.get_block_id(start_block as u32, block_device) as usize,
                Arc::clone(block_device),
            )
            .lock()
            .read(0, |data_block: &Data_Block| {
                let src = &data_block[start % BLOCK_SIZE..strat % BLOCK_SIZE + block_read_size];
                dst.copy_from_slice(src);
            });
            raed_size += block_read_size;
            // 移动至下一个块
            if end_current_block == end {
                break;
            }
            start_block += 1;
            start = end_current_block;
        }
        read_size
    }

}

///
/// 数据块中的目录项
/// 
const NAME_LENGTH_LIMIT: usize = 27;

#[repr(C)]
pub struct DirEntry {
    name: [u8; NAME_LENGTH_LIMIT],
    inode_number: u32,
}

pub const DIRENT_SIZE: usize = 32;

impl DirEntry {
    pub fn empty() -> Self {
        Self {
            name: [u8; NAME_LENGTH_LIMIT + 1], // add '\0'
            inode_number: 0, 
        }
    }

    pub fn new(name: &str, inode_number: u32) -> Self {
        let mut bytes = [0u8; NAME_LENGTH_LIMIT + 1];
        &mut bytes[..name.len()].copy_from_slice(name.as_bytes());
        Self {
            name: bytes,
            inode_number,
        }
    }
    
    // 读写目录数据
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const _ as usize as *const u8,
                DIRENT_SIZE,
            )
        }
    }
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self as *mut _ as usize as *mut u8,
                DIRENT_SIZE,
            )
        }
    }
    // 通过 name 和 inode_number 方法获取目录中的内容
    pub fn name(&self) -> &str {
        let len = (0usize..).find(|i| self.name[*i] == 0).unwrap();
        core::str::from_utf8(&self.name[..len]).unwrap()
    }
    pub fn inode_number(&self) -> u32 {
        self.inode_number
    }
}