pub mod block;
pub mod ext4;

pub fn init() {
    block::init();
    ext4::init();
}
