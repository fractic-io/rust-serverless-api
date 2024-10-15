use aws_lambda_events::apigw::ApiGatewayProxyRequest;
use fractic_generic_server_error::GenericServerError;

use crate::{
    auth::{get_sub_of_authenticated_user, is_admin, is_authenticated},
    errors::InvalidRequestError,
};

#[derive(Debug, Clone)]
pub struct RequestMetadata {
    pub is_authenticated: bool,
    pub is_admin: bool,
    pub user_sub: Option<String>,
}

// API Gateway request utils.
// --------------------------------------------------

pub fn parse_request_data<T>(request: &ApiGatewayProxyRequest) -> Result<T, GenericServerError>
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

pub fn parse_request_metadata(
    request: &ApiGatewayProxyRequest,
) -> Result<RequestMetadata, GenericServerError> {
    let is_authenticated = is_authenticated(request);
    Ok(RequestMetadata {
        is_authenticated,
        is_admin: is_admin(request),
        user_sub: if is_authenticated {
            Some(get_sub_of_authenticated_user(request)?)
        } else {
            None
        },
    })
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
        let result = parse_request_data::<TestData>(&request);
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
        let result = parse_request_data::<TestData>(&request);
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid request: missing request body."
        );
    }

    #[test]
    fn test_parse_request_invalid_json() {
        let request = ApiGatewayProxyRequest {
            body: Some("{invalid}".to_string()),
            ..Default::default()
        };
        let result = parse_request_data::<TestData>(&request);
        assert!(format!("{:?}", result.unwrap_err()).contains("InvalidRequestError"));
    }

    #[test]
    fn test_parse_request_valid_json_wrong_type() {
        let request = ApiGatewayProxyRequest {
            body: Some("{\"different_key\":\"value\"}".to_string()),
            ..Default::default()
        };
        let result = parse_request_data::<TestData>(&request);
        assert!(format!("{:?}", result.unwrap_err()).contains("InvalidRequestError"));
    }
}
