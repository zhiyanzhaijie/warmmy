pub(crate) fn db_err(err: toasty::Error) -> app::app_error::AppError {
    app::app_error::AppError::database(err)
}
