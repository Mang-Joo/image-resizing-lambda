mod clients;
mod error;
mod handler;
mod models;
mod services;

use aws_lambda_events::s3::S3Event;
use lambda_runtime::{Error, LambdaEvent, run, service_fn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize structured logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    tracing::info!("Lambda function initialized");

    // Run the Lambda handler
    run(service_fn(function_handler)).await
}

async fn function_handler(event: LambdaEvent<S3Event>) -> Result<(), Error> {
    handler::handle_s3_event(event.payload).await?;
    Ok(())
}
