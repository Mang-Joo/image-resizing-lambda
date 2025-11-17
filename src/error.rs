use thiserror::Error;

#[derive(Error, Debug)]
pub enum ImageResizeError {
    #[error("S3 error: {0}")]
    S3Error(String),

    #[error("Image processing error: {0}")]
    ImageError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error(
        "Image exceeds maximum size after {attempts} resize attempts (final size: {final_size} bytes)"
    )]
    SizeExceeded { attempts: u32, final_size: u64 },

    #[error("Invalid image format or corrupted data")]
    InvalidImage,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl From<aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::get_object::GetObjectError>>
    for ImageResizeError
{
    fn from(
        err: aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::get_object::GetObjectError>,
    ) -> Self {
        ImageResizeError::S3Error(format!("GetObject failed: {}", err))
    }
}

impl From<aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::put_object::PutObjectError>>
    for ImageResizeError
{
    fn from(
        err: aws_sdk_s3::error::SdkError<aws_sdk_s3::operation::put_object::PutObjectError>,
    ) -> Self {
        ImageResizeError::S3Error(format!("PutObject failed: {}", err))
    }
}

impl From<image::ImageError> for ImageResizeError {
    fn from(err: image::ImageError) -> Self {
        ImageResizeError::ImageError(err.to_string())
    }
}
