pub(crate) fn db_err(err: toasty::Error) -> application::app_error::AppError {
    application::app_error::AppError::database(err)
}
