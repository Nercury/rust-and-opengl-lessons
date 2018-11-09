use render_gl::{AabbMarker, DebugLines};
use selection::Selectables;

pub struct RenderSelectables {
    hover: Option<AabbMarker>,
    selected: Option<AabbMarker>,
}

impl RenderSelectables {
    pub fn new() -> RenderSelectables {
        RenderSelectables {
            hover: None,
            selected: None,
        }
    }

    pub fn update(&mut self, selectables: &Selectables, debug_lines: &DebugLines) {
        let selected = selectables.get_selected_aabb();

        self.selected = match (selected.clone(), self.selected.take()) {
            (Some((_, c)), None) => {
                Some(debug_lines.aabb_marker(c.isometry, c.aabb, [1.0, 1.0, 1.0, 1.0].into()))
            }
            (Some((_, c)), Some(item)) => {
                item.update_isometry(c.isometry);
                Some(item)
            }
            _ => None,
        };

        // do not show hover on selected items
        let hover = match (selected, selectables.get_hover_aabb()) {
            (Some((selected_handle, _)), Some((hover_handle, hover_aabb))) => {
                if selected_handle == hover_handle {
                    None
                } else {
                    Some((hover_handle, hover_aabb))
                }
            }
            (_, hover) => hover,
        };

        self.hover = match (hover, self.hover.take()) {
            (Some((_, c)), None) => {
                Some(debug_lines.aabb_marker(c.isometry, c.aabb, [1.0, 1.0, 1.0, 0.3].into()))
            }
            (Some((_, c)), Some(item)) => {
                item.update_isometry(c.isometry);
                Some(item)
            }
            _ => None,
        };
    }
}
