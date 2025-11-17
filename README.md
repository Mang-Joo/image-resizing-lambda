# Image Resizing Lambda (Rust)

AWS Lambda function for automatic image resizing with S3 triggers and DynamoDB metadata storage.

## Features

- ✅ S3-triggered automatic image processing
- ✅ Preserves aspect ratio during resize
- ✅ Ensures final image < 5 MB
- ✅ Skips processing if already < 5 MB
- ✅ DynamoDB metadata storage (optional)
- ✅ Structured JSON logging
- ✅ Configurable via environment variables

## Architecture

```
src/
├── main.rs              # Lambda entry point
├── handler.rs           # Event orchestration
├── error.rs             # Error types
├── clients/             # AWS SDK clients
│   ├── s3.rs           # S3 operations
│   └── dynamodb.rs     # DynamoDB operations
├── services/            # Business logic
│   └── image_processor.rs
└── models/              # Data structures
    ├── config.rs
    └── metadata.rs
```

## Prerequisites

- Rust 1.75+ (edition 2024)
- [cargo-lambda](https://www.cargo-lambda.info/)
- AWS CLI configured

## Installation

```bash
# Install cargo-lambda
brew install cargo-lambda  # macOS
# or
pip install cargo-lambda   # Cross-platform
```

## Configuration

Copy `.env.example` to `.env` and configure:

```bash
# Required
SOURCE_BUCKET=my-source-bucket

# Optional
DEST_BUCKET=my-dest-bucket              # Default: same as SOURCE_BUCKET
RESIZED_PREFIX=resized/                 # Default: resized/
MAX_WIDTH=1920                          # Default: 1920
MAX_HEIGHT=1080                         # Default: 1080
MAX_FILE_SIZE_BYTES=5242880            # Default: 5MB

# DynamoDB (optional)
DYNAMODB_TABLE_NAME=image-metadata     # If set, saves metadata
```

## Development

```bash
# Format code
cargo fmt

# Lint
cargo clippy --all-targets --all-features

# Test
cargo test

# Build
cargo build
```

## Build for Lambda

```bash
# Build for ARM64 (recommended - cheaper & faster)
cargo lambda build --release --arm64

# Or for x86_64
cargo lambda build --release
```

## Deployment

### Option 1: Using cargo-lambda (Recommended)

```bash
cargo lambda deploy \
  --iam-role arn:aws:iam::ACCOUNT_ID:role/lambda-execution-role \
  --env-var SOURCE_BUCKET=my-source-bucket \
  --env-var DEST_BUCKET=my-dest-bucket \
  --env-var DYNAMODB_TABLE_NAME=image-metadata \
  --memory 1024 \
  --timeout 300 \
  --architecture arm64
```

### Option 2: Manual Deployment

```bash
# Build
cargo lambda build --release --arm64

# Deploy via AWS CLI
aws lambda create-function \
  --function-name image-resizing-lambda \
  --runtime provided.al2 \
  --handler bootstrap \
  --zip-file fileb://target/lambda/image-resizing-lambda/bootstrap.zip \
  --role arn:aws:iam::ACCOUNT_ID:role/lambda-execution-role \
  --environment Variables="{SOURCE_BUCKET=my-source-bucket,DEST_BUCKET=my-dest-bucket}" \
  --memory-size 1024 \
  --timeout 300 \
  --architectures arm64
```

## IAM Permissions

Lambda execution role needs:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": ["s3:GetObject"],
      "Resource": "arn:aws:s3:::SOURCE_BUCKET/*"
    },
    {
      "Effect": "Allow",
      "Action": ["s3:PutObject"],
      "Resource": "arn:aws:s3:::DEST_BUCKET/*"
    },
    {
      "Effect": "Allow",
      "Action": ["dynamodb:PutItem"],
      "Resource": "arn:aws:dynamodb:*:*:table/image-metadata"
    },
    {
      "Effect": "Allow",
      "Action": ["logs:CreateLogGroup", "logs:CreateLogStream", "logs:PutLogEvents"],
      "Resource": "arn:aws:logs:*:*:*"
    }
  ]
}
```

## S3 Trigger Setup

```bash
# Add S3 trigger
aws s3api put-bucket-notification-configuration \
  --bucket my-source-bucket \
  --notification-configuration '{
    "LambdaFunctionConfigurations": [{
      "LambdaFunctionArn": "arn:aws:lambda:REGION:ACCOUNT:function:image-resizing-lambda",
      "Events": ["s3:ObjectCreated:*"],
      "Filter": {
        "Key": {
          "FilterRules": [{
            "Name": "suffix",
            "Value": ".jpg"
          }]
        }
      }
    }]
  }'

# Grant S3 permission to invoke Lambda
aws lambda add-permission \
  --function-name image-resizing-lambda \
  --statement-id s3-trigger \
  --action lambda:InvokeFunction \
  --principal s3.amazonaws.com \
  --source-arn arn:aws:s3:::my-source-bucket
```

## DynamoDB Table

If using metadata storage, create table:

```bash
aws dynamodb create-table \
  --table-name image-metadata \
  --attribute-definitions \
    AttributeName=object_key,AttributeType=S \
    AttributeName=timestamp,AttributeType=S \
  --key-schema \
    AttributeName=object_key,KeyType=HASH \
    AttributeName=timestamp,KeyType=RANGE \
  --billing-mode PAY_PER_REQUEST
```

## Usage

1. Upload image to S3:
```bash
aws s3 cp photo.jpg s3://my-source-bucket/
```

2. Lambda automatically processes it

3. Retrieve resized image location from DynamoDB:
```bash
aws dynamodb query \
  --table-name image-metadata \
  --key-condition-expression "object_key = :key" \
  --expression-attribute-values '{":key":{"S":"photo.jpg"}}' \
  --query 'Items[0].resized_key.S'
```

4. Download resized image:
```bash
aws s3 cp s3://my-dest-bucket/resized/1920x1080/photo.jpg ./
```

## Metadata Schema

DynamoDB stores:
- `object_key` (partition key) - Original filename
- `timestamp` (sort key) - Processing time
- `resized_key` - Path to resized image
- `original_size`, `final_size` - File sizes in bytes
- `original_width`, `original_height` - Original dimensions
- `final_width`, `final_height` - Final dimensions
- `was_resized` - Boolean flag

## Monitoring

View logs in CloudWatch:
```bash
aws logs tail /aws/lambda/image-resizing-lambda --follow
```

## Dependencies

- `lambda_runtime` 1.0.1 - Lambda runtime
- `aws-sdk-s3` 1.112.0 - S3 client
- `aws-sdk-dynamodb` 1.98.0 - DynamoDB client
- `image` 0.25.9 - Image processing
- `tokio` 1.48.0 - Async runtime
- `tracing` 0.1.41 - Structured logging

## License

MIT
