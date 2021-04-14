use thruster::errors::ThrusterError as Error;

use crate::context::Ctx;

pub trait ErrorSet {
    fn parsing_error(context: Ctx, error: &str) -> Error<Ctx>;
    fn generic_error(context: Ctx) -> Error<Ctx>;
    fn unauthorized_error(context: Ctx) -> Error<Ctx>;
    fn not_found_error(context: Ctx) -> Error<Ctx>;
    fn container_is_full(context: Ctx) -> Error<Ctx>;
}

impl ErrorSet for Error<Ctx> {
    fn parsing_error(context: Ctx, error: &str) -> Error<Ctx> {
        Error {
            context,
            message: format!("Failed to parse '{}'", error),
            status: 400,
            cause: None,
        }
    }

    fn generic_error(context: Ctx) -> Error<Ctx> {
        Error {
            context,
            message: "Something didn't work!".to_string(),
            status: 400,
            cause: None,
        }
    }

    fn unauthorized_error(context: Ctx) -> Error<Ctx> {
        Error {
            context,
            message: "Unauthorized".to_string(),
            status: 401,
            cause: None,
        }
    }

    fn not_found_error(context: Ctx) -> Error<Ctx> {
        Error {
            context,
            message: "Not found".to_string(),
            status: 404,
            cause: None,
        }
    }

    fn container_is_full(context: Ctx) -> Error<Ctx> {
        Error {
            context,
            message: "Destination was full".to_string(),
            status: 409,
            cause: None,
        }
    }
}
