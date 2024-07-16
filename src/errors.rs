use fractic_generic_server_error::{
    define_user_visible_error_type, define_user_visible_error_type_with_visible_info,
    GenericServerError, GenericServerErrorTrait,
};

define_user_visible_error_type_with_visible_info!(
    InvalidRequestError,
    "Request format was invalid: {user_visible_info}."
);
define_user_visible_error_type!(InvalidRouteError, "Route does not exist.");
define_user_visible_error_type!(UnauthorizedError, "Not authorized to perform this action.");
