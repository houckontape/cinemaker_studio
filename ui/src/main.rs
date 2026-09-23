pub mod adapters {
    pub mod camera_adapter;
}
pub mod components {
    pub mod camera_panel;
}
pub mod liveview_panel;
pub mod workspace_app;

use workspace_app::WorkspaceApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Cinemaker Studio",
        options,
        Box::new(|_cc| Ok(Box::<WorkspaceApp>::default())),
    )
}
