use ratatui::Frame;

use crate::app::App;

use super::container::{render_container_in_area, render_list};
use super::context::RenderContext;
use super::header::render_header_bar;
use super::layout::layout_nodes;
use super::overlay::{OverlayLayout, render_overlays};
use super::prep::prepare_render;
use super::scroll::set_edit_cursor;
use super::view_model::{PanelContent, build_view_model};

pub fn render_app(f: &mut Frame<'_>, app: &mut App) {
    let context = RenderContext::new(f.area());
    let prep = prepare_render(app, &context);

    if let Some(cursor) = prep.edit_cursor.as_ref() {
        set_edit_cursor(
            f,
            cursor,
            app.scroll_offset_mut(),
            context.scroll_height,
            context.content_area,
        );
    }

    let view_model = build_view_model(app, &context, prep);

    render_header_bar(f, context.header_area, view_model.header);

    for (panel_id, rect) in layout_nodes(context.main_area, &view_model.layout) {
        if let Some(panel) = view_model.panels.get(panel_id) {
            let focused = view_model.focused_panel == Some(panel_id);
            let container_layout = render_container_in_area(f, rect, &panel.config, focused);
            match &panel.content {
                PanelContent::EntryList(list) => {
                    render_list(f, list.clone(), &container_layout);
                }
            }
        }
    }

    render_overlays(
        f,
        view_model.overlays,
        OverlayLayout {
            screen_area: context.size,
        },
    );
}
