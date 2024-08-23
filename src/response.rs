use aws_lambda_events::apigw::ApiGatewayProxyResponse;
use fractic_generic_server_error::GenericServerError;
use lambda_runtime::Error;
use serde::Serialize;

// API Gateway response utils.
// --------------------------------------------------

// All API responses are wrapped in the following wrapper:
#[derive(Debug, Serialize)]
struct ResponseWrapper<T> {
    ok: bool,
    // If OK, response data.
    data: Option<T>,
    // If not OK, error message safe to show to user.
    error: Option<String>,
}

pub fn build_result<T>(data: T) -> Result<ApiGatewayProxyResponse, Error>
where
    T: serde::Serialize,
{
    let payload = ResponseWrapper {
        ok: true,
        data: Some(data),
        error: None,
    };
    let resp = ApiGatewayProxyResponse {
        status_code: 200,
        headers: Default::default(),
        multi_value_headers: Default::default(),
        body: Some(serde_json::to_string(&payload)?.into()),
        is_base64_encoded: false,
    };
    Ok(resp)
}

pub fn build_error(error: GenericServerError) -> Result<ApiGatewayProxyResponse, Error> {
    if error.should_be_shown_to_client() {
        // Since the data field will be set to None, we need to specify the
        // correct type T, so just use int.
        let payload = ResponseWrapper::<i8> {
            ok: false,
            data: None,
            error: Some(error.to_string().into()),
        };
        let resp = ApiGatewayProxyResponse {
            // Outer status code should still be 200 for client-errors,
            // otherwise Amplify will treat it as a server error. The client
            // will know there is a client error because ok == false.
            status_code: 200,
            headers: Default::default(),
            multi_value_headers: Default::default(),
            body: Some(serde_json::to_string(&payload)?.into()),
            is_base64_encoded: false,
        };
        Ok(resp)
    } else {
        // Return internal server error (500).
        Err(error.into_std_error().into())
    }
}

// Tests.
// --------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::InvalidRequestError;
    use aws_lambda_events::encodings::Body;
    use fractic_generic_server_error::common::CriticalError;
    use serde_json::Value;

    #[derive(Debug, Serialize)]
    struct MockResponseData {
        key: String,
    }

    #[test]
    fn test_build_result_string() {
        let data = "Test string.".to_string();
        let result = build_result(data).unwrap();
        let body: Value = serde_json::from_str(match &result.body.unwrap() {
            Body::Text(b) => b,
            _ => panic!("Expected response body."),
        })
        .unwrap();

        assert_eq!(result.status_code, 200);
        assert_eq!(body["ok"].as_bool().unwrap(), true);
        assert_eq!(body["data"].as_str().unwrap(), "Test string.");
        assert_eq!(body["error"].is_null(), true);
    }

    #[test]
    fn test_build_result_object() {
        let error = MockResponseData {
            key: "Test value.".to_string(),
        };
        let result = build_result(error).unwrap();
        let body: Value = serde_json::from_str(match &result.body.unwrap() {
            Body::Text(b) => b,
            _ => panic!("Expected response body."),
        })
        .unwrap();

        assert_eq!(result.status_code, 200);
        assert_eq!(body["ok"].as_bool().unwrap(), true);
        assert_eq!(body["data"]["key"].as_str().unwrap(), "Test value.");
        assert_eq!(body["error"].is_null(), true);
    }

    #[test]
    fn test_build_error_shown_to_client() {
        let error = InvalidRequestError::new("cxt", "msg", "user visible info".to_string());
        let result = build_error(error).unwrap();
        let body: Value = serde_json::from_str(match &result.body.unwrap() {
            Body::Text(b) => b,
            _ => panic!("Expected response body."),
        })
        .unwrap();

        assert_eq!(result.status_code, 200);
        assert_eq!(body["ok"].as_bool().unwrap(), false);
        assert_eq!(body["data"].is_null(), true);
        assert_eq!(
            body["error"].as_str().unwrap(),
            "Invalid request: user visible info."
        );
    }

    #[test]
    fn test_build_error_not_shown_to_client() {
        let error = CriticalError::new("cxt", "msg");
        let result = build_error(error);
        assert_eq!(result.is_err(), true);
    }
}
