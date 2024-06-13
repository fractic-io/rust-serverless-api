use aws_lambda_events::apigw::ApiGatewayProxyRequest;
use fractic_generic_server_error::GenericServerError;

use crate::errors::InvalidRequestError;

// API Gateway request utils.
// --------------------------------------------------

pub fn parse_request<T>(request: &ApiGatewayProxyRequest) -> Result<T, GenericServerError>
where
    T: serde::de::DeserializeOwned,
{
    let dbg_cxt: &'static str = "parse_request";
    let body = match &request.body {
        Some(b) => b,
        None => {
            return Err(InvalidRequestError::new(
                dbg_cxt,
                "",
                "missing request body".to_string(),
            ));
        }
    };
    serde_json::from_str(body).map_err(|e| InvalidRequestError::new(dbg_cxt, "", e.to_string()))
}

// Tests.
// --------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize, Debug, PartialEq)]
    struct TestData {
        key: String,
    }

    #[test]
    fn test_parse_request() {
        let request = ApiGatewayProxyRequest {
            body: Some("{\"key\":\"value\"}".to_string()),
            ..Default::default()
        };
        let result = parse_request::<TestData>(&request);
        assert_eq!(
            result.unwrap(),
            TestData {
                key: "value".to_string()
            }
        );
    }

    #[test]
    fn test_parse_request_missing_body() {
        let request = ApiGatewayProxyRequest {
            body: None,
            ..Default::default()
        };
        let result = parse_request::<TestData>(&request);
        assert_eq!(
            result.unwrap_err().to_string(),
            "Request format was invalid: missing request body."
        );
    }

    #[test]
    fn test_parse_request_invalid_json() {
        let request = ApiGatewayProxyRequest {
            body: Some("{invalid}".to_string()),
            ..Default::default()
        };
        let result = parse_request::<TestData>(&request);
        assert!(format!("{:?}", result.unwrap_err()).contains("InvalidRequestError"));
    }

    #[test]
    fn test_parse_request_valid_json_wrong_type() {
        let request = ApiGatewayProxyRequest {
            body: Some("{\"different_key\":\"value\"}".to_string()),
            ..Default::default()
        };
        let result = parse_request::<TestData>(&request);
        assert!(format!("{:?}", result.unwrap_err()).contains("InvalidRequestError"));
    }
}
