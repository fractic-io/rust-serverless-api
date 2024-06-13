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
    ($handler_name:ident, $func_name:ident, $request_data_type:ident) => {
        pub async fn $handler_name(
            event: LambdaEvent<ApiGatewayProxyRequest>,
        ) -> Result<ApiGatewayProxyResponse, Error> {
            let request_parsing = parse_request::<$request_data_type>(&event.payload);
            if request_parsing.is_ok() {
                match $func_name(request_parsing.unwrap()).await {
                    Ok(result) => build_result(result),
                    Err(error) => build_error(error),
                }
            } else {
                build_error(request_parsing.unwrap_err())
            }
        }
    };
    ($handler_name:ident, $func_name:ident) => {
        pub async fn $handler_name(
            event: LambdaEvent<ApiGatewayProxyRequest>,
        ) -> Result<ApiGatewayProxyResponse, Error> {
            match $func_name().await {
                Ok(result) => build_result(result),
                Err(error) => build_error(error),
            }
        }
    };
}
