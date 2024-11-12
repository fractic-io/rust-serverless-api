use fractic_server_error::{define_client_error, define_sensitive_error};

define_client_error!(InvalidRequestError, "Request is invalid: {details}.", { details: &str });
define_client_error!(InvalidRouteError, "Route '{route:?}' does not exist.", { route: Option<String> });
define_sensitive_error!(UnauthorizedError, "Not authorized to access this resource.");
