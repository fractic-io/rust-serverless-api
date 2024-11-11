use fractic_generic_server_error::{
    define_internal_error_type, define_user_visible_error_type, GenericServerError,
    GenericServerErrorTrait,
};

define_internal_error_type!(InvalidRequestError, "Request is invalid.");
define_internal_error_type!(InvalidRouteError, "Route does not exist.");
define_user_visible_error_type!(UnauthorizedError, "Not authorized to access this resource.");
