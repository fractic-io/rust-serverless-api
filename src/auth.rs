use aws_lambda_events::apigw::ApiGatewayProxyRequest;
use fractic_generic_server_error::{common::CriticalError, GenericServerError};

use crate::errors::UnauthorizedError;

// API Gateway authentication utils.
// --------------------------------------------------

pub fn is_authenticated(req: &ApiGatewayProxyRequest) -> bool {
    match req.request_context.authorizer.get("claims") {
        Some(claims) => match claims.get("cognito:username") {
            Some(_) => true,
            None => false,
        },
        None => false,
    }
}

pub fn is_admin(req: &ApiGatewayProxyRequest) -> bool {
    match req.request_context.authorizer.get("claims") {
        Some(claims) => match claims.get("cognito:groups") {
            Some(groups_val) => match groups_val.as_str() {
                Some(groups_str) => groups_str.split(',').any(|g| g == "admin"),
                None => false,
            },
            None => false,
        },
        None => false,
    }
}

pub fn get_sub_of_authenticated_user(
    req: &ApiGatewayProxyRequest,
) -> Result<String, GenericServerError> {
    let dbg_cxt: &'static str = "get_sub_of_authenticated_user";
    match req.request_context.authorizer.get("claims") {
        Some(claims) => match claims.get("sub") {
            Some(sub) => match sub.as_str() {
                Some(sub_str) => Ok(sub_str.into()),
                // Unexpected, so throw a Critical error.
                None => Err(CriticalError::new(
                    dbg_cxt,
                    "authorizer claims sub was not a string",
                )),
            },
            // Unexpected, so throw a Critical error.
            None => Err(CriticalError::new(
                dbg_cxt,
                "authorizer claims did not contain sub",
            )),
        },
        None => Err(UnauthorizedError::new(
            dbg_cxt,
            "authorizer did not contain any claims",
        )),
    }
}

// Tests.
// --------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use aws_lambda_events::apigw::{ApiGatewayProxyRequest, ApiGatewayProxyRequestContext};

    fn create_authenticated_request() -> ApiGatewayProxyRequest {
        ApiGatewayProxyRequest {
            request_context: ApiGatewayProxyRequestContext {
                authorizer: [(
                    "claims".into(),
                    serde_json::json!({
                        "cognito:username": "FakeUsername",
                        "sub": "FakeUserSub"
                    }),
                )]
                .into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn create_unauthenticated_request() -> ApiGatewayProxyRequest {
        ApiGatewayProxyRequest {
            request_context: ApiGatewayProxyRequestContext {
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn test_is_authenticated() {
        let authenticated_request = create_authenticated_request();
        let unauthenticated_request = create_unauthenticated_request();

        assert!(is_authenticated(&authenticated_request));
        assert!(!is_authenticated(&unauthenticated_request));
    }

    #[test]
    fn test_get_sub_of_authenticated_user() {
        let authenticated_request = create_authenticated_request();
        let unauthenticated_request = create_unauthenticated_request();

        assert_eq!(
            get_sub_of_authenticated_user(&authenticated_request).unwrap(),
            "FakeUserSub".to_string()
        );
        assert!(get_sub_of_authenticated_user(&unauthenticated_request).is_err());
    }
}
