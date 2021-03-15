use tracing::error;

pub fn log_and_return_error_string(error_string: String) -> String {
    error!("{}", error_string);
    error_string
}
