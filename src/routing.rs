use std::collections::HashMap;

use aws_lambda_events::{
    apigw::{ApiGatewayProxyRequest, ApiGatewayProxyResponse},
    http::Method,
};
use core::future::Future;
use lambda_runtime::{Error, LambdaEvent};
use std::pin::Pin;

use crate::errors::{InvalidRouteError, UnauthorizedError};

use super::{
    auth::{is_admin, is_authenticated},
    response::build_error,
};

// API Gateway routing config.
// --------------------------------------------------

pub enum AccessLevel {
    Guest,
    User,
    Admin,
    None,
}

type RouteHandler = Box<
    dyn Fn(
        LambdaEvent<ApiGatewayProxyRequest>,
    ) -> Pin<Box<dyn Future<Output = Result<ApiGatewayProxyResponse, Error>>>>,
>;

pub struct FunctionRoute {
    pub access_level: AccessLevel,
    pub handler: RouteHandler,
}

pub struct CrudRoute {
    pub create_access_level: AccessLevel,
    pub read_access_level: AccessLevel,
    pub update_access_level: AccessLevel,
    pub delete_access_level: AccessLevel,
    pub handler: RouteHandler,
}

pub struct RoutingConfig {
    pub function_routes: HashMap<String, FunctionRoute>,
    pub crud_routes: HashMap<String, CrudRoute>,
}

// API Gateway routing utils.
// --------------------------------------------------

pub fn box_route_handler<T>(f: fn(LambdaEvent<ApiGatewayProxyRequest>) -> T) -> RouteHandler
where
    T: Future<Output = Result<ApiGatewayProxyResponse, Error>> + 'static,
{
    Box::new(move |e| Box::pin(f(e)))
}

fn find_function_route<'a>(
    config: &'a RoutingConfig,
    event: &LambdaEvent<ApiGatewayProxyRequest>,
) -> Option<(&'a RouteHandler, &'a AccessLevel)> {
    let method = &event.payload.http_method;
    if method == Method::POST {
        event
            .payload
            .path_parameters
            .get("proxy")
            .and_then(|proxy| config.function_routes.get(proxy))
            .map(|route| (&route.handler, &route.access_level))
    } else {
        None
    }
}

fn find_crud_route<'a>(
    config: &'a RoutingConfig,
    event: &LambdaEvent<ApiGatewayProxyRequest>,
) -> Option<(&'a RouteHandler, &'a AccessLevel)> {
    let method = &event.payload.http_method;
    event
        .payload
        .path_parameters
        .get("proxy")
        .and_then(|proxy| config.crud_routes.get(proxy))
        .map(|route| {
            (
                &route.handler,
                match method {
                    &Method::POST => &route.create_access_level,
                    &Method::GET => &route.read_access_level,
                    &Method::PUT => &route.update_access_level,
                    &Method::DELETE => &route.delete_access_level,
                    _ => &AccessLevel::None,
                },
            )
        })
}

pub async fn handle_route(
    config: RoutingConfig,
    event: LambdaEvent<ApiGatewayProxyRequest>,
) -> Result<ApiGatewayProxyResponse, Error> {
    let dbg_cxt: &'static str = "handle_route";
    let is_logged_in = is_authenticated(&event.payload);
    let is_admin = is_admin(&event.payload);

    let route_search =
        find_function_route(&config, &event).or_else(|| find_crud_route(&config, &event));
    let (handler, access_level) = match route_search {
        Some((handler, access_level)) => (handler, access_level),
        None => return build_error(InvalidRouteError::new(dbg_cxt, "Route does not exist.")),
    };

    let is_authenticated_for_route = match access_level {
        AccessLevel::Guest => true,
        AccessLevel::User => is_logged_in,
        AccessLevel::Admin => is_logged_in && is_admin,
        AccessLevel::None => false,
    };

    if is_authenticated_for_route {
        handler(event).await
    } else {
        build_error(UnauthorizedError::new(
            dbg_cxt,
            "Not authorized to access route.",
        ))
    }
}
