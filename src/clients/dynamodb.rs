use crate::error::ImageResizeError;
use crate::models::metadata::ImageMetadata;
use aws_sdk_dynamodb::{Client, types::AttributeValue};
use std::collections::HashMap;

pub struct DynamoDbClient {
    client: Client,
}

impl DynamoDbClient {
    pub async fn new() -> Self {
        let config = aws_config::load_from_env().await;
        
        if let Some(region) = config.region() {
            tracing::info!(region = %region, "Initializing DynamoDB client");
        } else {
            tracing::warn!("DynamoDB client initialized without explicit region");
        }

        let client = Client::new(&config);
        Self { client }
    }

    pub async fn save_metadata(
        &self,
        table_name: &str,
        metadata: ImageMetadata,
    ) -> Result<(), ImageResizeError> {
        let mut item = HashMap::new();

        item.insert(
            "object_key".to_string(),
            AttributeValue::S(metadata.object_key.clone()),
        );
        item.insert(
            "timestamp".to_string(),
            AttributeValue::S(metadata.timestamp),
        );
        item.insert(
            "resized_key".to_string(),
            AttributeValue::S(metadata.resized_key),
        );
        item.insert(
            "original_size".to_string(),
            AttributeValue::N(metadata.original_size.to_string()),
        );
        item.insert(
            "final_size".to_string(),
            AttributeValue::N(metadata.final_size.to_string()),
        );
        item.insert(
            "original_width".to_string(),
            AttributeValue::N(metadata.original_width.to_string()),
        );
        item.insert(
            "original_height".to_string(),
            AttributeValue::N(metadata.original_height.to_string()),
        );
        item.insert(
            "final_width".to_string(),
            AttributeValue::N(metadata.final_width.to_string()),
        );
        item.insert(
            "final_height".to_string(),
            AttributeValue::N(metadata.final_height.to_string()),
        );
        item.insert(
            "was_resized".to_string(),
            AttributeValue::Bool(metadata.was_resized),
        );

        self.client
            .put_item()
            .table_name(table_name)
            .set_item(Some(item))
            .send()
            .await
            .map_err(|e| ImageResizeError::S3Error(format!("DynamoDB put_item failed: {}", e)))?;

        tracing::info!(
            table = %table_name,
            object_key = %metadata.object_key,
            "Saved metadata to DynamoDB"
        );

        Ok(())
    }
}
