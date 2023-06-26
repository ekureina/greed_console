use log::error;
use std::time::Duration;

pub fn error_log_and_notify<M: Into<String>>(toasts_ref: &mut egui_notify::Toasts, message: M) {
    let error = message.into();
    error!("{error}");
    toasts_ref.dismiss_oldest_toast();
    toasts_ref
        .error(error)
        .set_closable(true)
        .set_duration(Some(Duration::from_secs(20)));
}
