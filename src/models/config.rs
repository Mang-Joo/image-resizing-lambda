use crate::error::ImageResizeError;
use std::env;

pub const MAX_FILE_SIZE_BYTES: u64 = 5 * 1024 * 1024; // 5 MB

#[derive(Debug, Clone)]
pub struct Config {
    pub dest_bucket: String,
    pub resized_prefix: String,
    pub max_width: u32,
    pub max_height: u32,
    pub max_file_size: u64,
    pub dynamodb_table: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, ImageResizeError> {
        let dest_bucket = env::var("DEST_BUCKET")
            .map_err(|_| ImageResizeError::ConfigError("DEST_BUCKET not set".into()))?;

        Ok(Config {
            dest_bucket,
            resized_prefix: env::var("RESIZED_PREFIX").unwrap_or_else(|_| "resized/".into()),
            max_width: env::var("MAX_WIDTH")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1920),
            max_height: env::var("MAX_HEIGHT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1080),
            max_file_size: env::var("MAX_FILE_SIZE_BYTES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(MAX_FILE_SIZE_BYTES),
            dynamodb_table: env::var("DYNAMODB_TABLE_NAME").ok(),
        })
    }
}
