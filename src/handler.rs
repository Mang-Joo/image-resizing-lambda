use crate::{
    clients::{dynamodb::DynamoDbClient, s3::S3Client},
    error::ImageResizeError,
    models::{config::Config, metadata::ImageMetadata},
    services::image_processor::ImageProcessor,
};
use aws_lambda_events::s3::S3Event;

pub async fn handle_s3_event(event: S3Event) -> Result<(), ImageResizeError> {
    let config = Config::from_env()?;
    let s3_client = S3Client::new().await;
    let image_processor =
        ImageProcessor::new(config.max_width, config.max_height, config.max_file_size);

    let dynamodb_client = if config.dynamodb_table.is_some() {
        Some(DynamoDbClient::new().await)
    } else {
        None
    };

    for record in event.records {
        let bucket = record
            .s3
            .bucket
            .name
            .ok_or_else(|| ImageResizeError::S3Error("Missing bucket name".into()))?;
        let raw_key = record
            .s3
            .object
            .key
            .ok_or_else(|| ImageResizeError::S3Error("Missing object key".into()))?;

        let key = urlencoding::decode(&raw_key)
            .map_err(|e| ImageResizeError::S3Error(format!("Failed to decode key: {}", e)))?
            .into_owned();

        tracing::info!(
            bucket = %bucket,
            key = %key,
            event_name = %record.event_name.unwrap_or_default(),
            "Processing S3 event"
        );

        // Fetch original image
        let (image_bytes, original_size) = s3_client.get_object(&bucket, &key).await?;

        // Process image
        let result = image_processor.process_image(image_bytes, original_size)?;

        // Generate destination key
        // Logic: Strip 'source_prefix' (e.g. "original/") from key, prepend 'resized_prefix' (e.g. "resized/")
        // Example: "original/post-1/img.jpg" -> "post-1/img.jpg" -> "resized/post-1/img.jpg"
        let relative_key = if key.starts_with(&config.source_prefix) {
            key.strip_prefix(&config.source_prefix).unwrap_or(&key)
        } else {
            &key
        };
        // Remove leading slashes to prevent "//"
        let clean_key = relative_key.trim_start_matches('/');

        let dest_key = format!("{}{}", config.resized_prefix, clean_key);

        let final_size = result.data.len() as u64;

        // Upload processed image
        s3_client
            .put_object(&config.dest_bucket, &dest_key, result.data, "image/jpeg")
            .await?;

        // Save metadata to DynamoDB if configured
        if let (Some(client), Some(table_name)) = (&dynamodb_client, &config.dynamodb_table) {
            let metadata = ImageMetadata {
                object_key: key.clone(),
                resized_key: dest_key.clone(),
                original_size,
                final_size,
                original_width: result.original_width,
                original_height: result.original_height,
                final_width: result.width,
                final_height: result.height,
                was_resized: result.was_resized,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };

            client.save_metadata(table_name, metadata).await?;
        } else {
            tracing::warn!("DynamoDB table not configured, skipping metadata save");
        }

        tracing::info!(
            original_bucket = %bucket,
            original_key = %key,
            dest_bucket = %config.dest_bucket,
            dest_key = %dest_key,
            original_size = original_size,
            final_size = final_size,
            was_resized = result.was_resized,
            final_width = result.width,
            final_height = result.height,
            "Successfully processed image"
        );
    }

    Ok(())
}
