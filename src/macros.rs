#[macro_export]
macro_rules! aws_lambda {
    ($handler:expr) => {
        #[tokio::main]
        async fn main() -> Result<(), Error> {
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::INFO)
                // Disable printing module name in every log line.
                .with_target(false)
                // Disable printing time since CloudWatch already logs ingestion time.
                .without_time()
                .init();

            lambda_runtime::run(service_fn($handler)).await
        }
    };
}

#[macro_export]
macro_rules! aws_lambda_from_routing_config {
    ($config:expr) => {
        #[tokio::main]
        async fn main() -> Result<(), Error> {
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::INFO)
                // Disable printing module name in every log line.
                .with_target(false)
                // Disable printing time since CloudWatch already logs ingestion time.
                .without_time()
                .init();

            lambda_runtime::run(service_fn(|e| handle_route($config, e))).await
        }
    };
}

#[macro_export]
macro_rules! register_function_route {
    ($handler_name:ident, $func:ident, $validator:ident, $request_data_type:ident) => {
        pub async fn $handler_name(
            event: LambdaEvent<ApiGatewayProxyRequest>,
            metadata: RequestMetadata,
        ) -> Result<ApiGatewayProxyResponse, Error> {
            match parse_request_data::<$request_data_type>(&event.payload) {
                Ok(obj) => match $validator(&obj, metadata) {
                    Ok(_) => match $func(obj).await {
                        Ok(result) => build_result(result),
                        Err(func_error) => build_error(func_error),
                    },
                    Err(validation_error) => build_error(validation_error),
                },
                Err(request_parsing_error) => build_error(request_parsing_error),
            }
        }
    };
    ($handler_name:ident, $func:ident, $validator:ident) => {
        pub async fn $handler_name(
            event: LambdaEvent<ApiGatewayProxyRequest>,
            metadata: RequestMetadata,
        ) -> Result<ApiGatewayProxyResponse, Error> {
            match $validator(metadata) {
                Ok(_) => match $func().await {
                    Ok(result) => build_result(result),
                    Err(func_error) => build_error(func_error),
                },
                Err(validation_error) => build_error(validation_error),
            }
        }
    };
}
