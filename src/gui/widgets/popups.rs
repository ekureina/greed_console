use egui::Window;

pub struct ErrorPopup<'a> {
    containing_window: Window<'a>,
    error_text: String,
}

impl<'a> ErrorPopup<'a> {
    pub fn new<T: Into<String>>(error_text: T) -> ErrorPopup<'a> {
        let window = Window::new("Error!")
            .collapsible(false)
            .resizable(false)
            .auto_sized();
        ErrorPopup {
            containing_window: window,
            error_text: error_text.into(),
        }
    }

    pub fn show(self, ctx: &egui::Context) -> bool {
        let mut should_close = false;
        self.containing_window.show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(self.error_text);
                if ui.button("Ok").clicked() {
                    should_close = true;
                }
            });
        });
        should_close
    }
}
