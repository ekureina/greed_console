use super::campaign::CampaignGui;

pub struct CampaignTabViewer<'a> {
    pub should_remove_gui: &'a mut bool,
}

impl<'a> egui_dock::TabViewer for CampaignTabViewer<'a> {
    type Tab = &'a mut CampaignGui;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        tab.ui(ui);
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab.get_path() {
            Some(text) => {
                format!(
                    "{} [{}]",
                    tab.get_save().get_campaign_name(),
                    text.to_string_lossy()
                )
            }
            None => tab.get_save().get_campaign_name(),
        }
        .into()
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
        *self.should_remove_gui = tab.save().is_some_and(|result| result.is_ok());
        *self.should_remove_gui
    }
}
