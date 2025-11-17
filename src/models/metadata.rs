#[derive(Debug)]
pub struct ImageMetadata {
    pub object_key: String,
    pub resized_key: String,
    pub original_size: u64,
    pub final_size: u64,
    pub original_width: u32,
    pub original_height: u32,
    pub final_width: u32,
    pub final_height: u32,
    pub was_resized: bool,
    pub timestamp: String,
}
