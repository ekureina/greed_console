use super::campaign::CampaignGui;

#[derive(Default, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct CampaignTabViewer {
    tabs_to_force_close: Vec<String>,
}

impl CampaignTabViewer {
    pub fn new() -> CampaignTabViewer {
        CampaignTabViewer::default()
    }

    pub fn set_tabs_to_close(&mut self, tabs: Vec<String>) {
        self.tabs_to_force_close = tabs;
    }
}

impl egui_dock::TabViewer for CampaignTabViewer {
    type Tab = CampaignGui;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        tab.ui(ui);
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        let dirty_mark = if tab.save_is_dirty() { "* " } else { "" };

        match tab.get_path() {
            Some(text) => {
                format!(
                    "{dirty_mark}{} [{}]",
                    tab.get_save().get_campaign_name(),
                    text.to_string_lossy()
                )
            }
            None => format!("{dirty_mark}{}", tab.get_save().get_campaign_name()),
        }
        .into()
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
        tab.save().is_some_and(|result| result.is_ok())
    }

    fn force_close(&mut self, tab: &mut Self::Tab) -> bool {
        self.tabs_to_force_close
            .iter()
            .position(|campaign_name| campaign_name.clone() == tab.get_save().get_campaign_name())
            .map(|index| self.tabs_to_force_close.swap_remove(index))
            .is_some()
    }
}
