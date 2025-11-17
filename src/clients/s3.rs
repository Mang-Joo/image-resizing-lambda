use crate::error::ImageResizeError;
use aws_sdk_s3::Client;
use aws_sdk_s3::primitives::ByteStream;

pub struct S3Client {
    client: Client,
}

impl S3Client {
    pub async fn new() -> Self {
        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);
        Self { client }
    }

    pub async fn get_object(
        &self,
        bucket: &str,
        key: &str,
    ) -> Result<(Vec<u8>, u64), ImageResizeError> {
        tracing::info!(bucket = %bucket, key = %key, "Fetching object from S3");

        let response = self
            .client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await?;

        let content_length = response.content_length().unwrap_or(0) as u64;
        let bytes = response
            .body
            .collect()
            .await
            .map_err(|e| {
                ImageResizeError::S3Error(format!("Failed to read S3 object body: {}", e))
            })?
            .into_bytes()
            .to_vec();

        tracing::info!(
            bucket = %bucket,
            key = %key,
            size_bytes = content_length,
            "Successfully fetched object"
        );

        Ok((bytes, content_length))
    }

    pub async fn put_object(
        &self,
        bucket: &str,
        key: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> Result<(), ImageResizeError> {
        tracing::info!(
            bucket = %bucket,
            key = %key,
            size_bytes = data.len(),
            content_type = %content_type,
            "Uploading object to S3"
        );

        self.client
            .put_object()
            .bucket(bucket)
            .key(key)
            .body(ByteStream::from(data))
            .content_type(content_type)
            .send()
            .await?;

        tracing::info!(bucket = %bucket, key = %key, "Successfully uploaded object");

        Ok(())
    }
}
