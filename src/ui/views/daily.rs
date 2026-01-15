use crate::app::App;
use crate::ui::container::view_content_container_config;
use crate::ui::context::RenderContext;
use crate::ui::daily::build_daily_list;
use crate::ui::layout::PanelId;
use crate::ui::theme;
use crate::ui::view_model::{PanelContent, PanelModel};

use super::{ViewSpec, list_panel_content_area};

pub fn build_daily_view_spec(app: &App, context: &RenderContext) -> ViewSpec {
    let list_config = view_content_container_config(theme::DAILY_PRIMARY);
    let list_content_width = list_content_width_for_daily(context);
    let list = build_daily_list(app, list_content_width);
    let list_panel = PanelModel::new(PanelId(0), list_config, PanelContent::EntryList(list));
    ViewSpec::single_panel(list_panel)
}

pub(crate) fn list_content_width_for_daily(context: &RenderContext) -> usize {
    list_panel_content_area(context, theme::DAILY_PRIMARY).width as usize
}

pub(crate) fn list_content_height_for_daily(context: &RenderContext) -> usize {
    list_panel_content_area(context, theme::DAILY_PRIMARY).height as usize
}
