use log::error;

pub fn error_log_and_popup<M: Into<String>>(error_text_ref: &mut Option<String>, error_message: M) {
    let message = error_message.into();
    error!("{message}");
    let _ = error_text_ref.insert(message);
}
