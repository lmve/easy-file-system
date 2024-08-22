///
/// 块设备接口层
/// 定义文件系统与块设备之间的接口
/// 

pub trait BlockDevice: Send + Sync + Any {
    fn read_block(&self, block_id:usize, buf: &mut [u8]);
    fn write_block(&self, block_id:usize, buf: &mut [u8]);
}
