use crate::app::App;
use crate::ui::container::{content_area_for, view_content_container_config};
use crate::ui::context::RenderContext;
use crate::ui::daily::build_daily_list;
use crate::ui::layout::PanelId;
use crate::ui::theme;
use crate::ui::view_model::{PanelContent, PanelModel};

use super::ViewSpec;

pub fn build_daily_view_spec(app: &App, context: &RenderContext) -> ViewSpec {
    let list_config = view_content_container_config(theme::BORDER_DAILY);
    let list_content_width = list_content_width_for_daily(context);
    let list = build_daily_list(app, list_content_width);
    let list_panel = PanelModel::new(PanelId(0), list_config, PanelContent::EntryList(list));
    ViewSpec::single_panel(list_panel)
}

pub(crate) fn list_content_width_for_daily(context: &RenderContext) -> usize {
    list_panel_content_area(context).width as usize
}

pub(crate) fn list_content_height_for_daily(context: &RenderContext) -> usize {
    list_panel_content_area(context).height as usize
}

fn list_panel_content_area(context: &RenderContext) -> ratatui::layout::Rect {
    content_area_for(
        context.content_area,
        &view_content_container_config(theme::BORDER_DAILY),
    )
}
