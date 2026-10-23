//! Native offline developer workbench.

mod actions;
mod buffer;
mod json_run;
mod inspector_workbench;
mod json_workbench;
mod text_field;
mod theme;
mod transform_workbench;
mod tree_state;
mod ui;
mod workspace;

use gpui::{
    App, Application, Bounds, TitlebarOptions, WindowBounds, WindowOptions, prelude::*, px, size,
};
use workspace::Workspace;

fn main() {
    Application::new().run(|cx: &mut App| {
        actions::bind(cx);
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();

        let bounds = Bounds::centered(None, size(px(1240.), px(780.)), cx);
        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_min_size: Some(size(px(860.), px(560.))),
                    titlebar: Some(titlebar_options()),
                    focus: true,
                    ..WindowOptions::default()
                },
                |_, cx| cx.new(Workspace::new),
            )
            .expect("open the main window");
        window
            .update(cx, |workspace, window, cx| {
                workspace.focus_editor(window, cx)
            })
            .expect("focus the JSON editor");
        cx.activate(true);
    });
}

fn titlebar_options() -> TitlebarOptions {
    #[cfg(target_os = "macos")]
    {
        use gpui::point;
        TitlebarOptions {
            title: Some("Devtools".into()),
            appears_transparent: true,
            traffic_light_position: Some(point(px(12.), px(14.))),
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        // Normal decorated window on Linux/Windows. No transparent titlebar
        // fusion and no traffic-light inset — those are macOS-only.
        TitlebarOptions {
            title: Some("Devtools".into()),
            appears_transparent: false,
            traffic_light_position: None,
        }
    }
}
