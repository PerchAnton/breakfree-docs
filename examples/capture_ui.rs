//! Development-only native screenshot harness; never part of the editor CLI.
use breakfree_docs::{
    gui::{Action, DocsApp},
    session::Session,
    storage,
};
use eframe::egui;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 3 || !["home", "file", "navigation", "formatting"].contains(&args[2].as_str())
    {
        return Err("Usage: capture_ui <width-DIP> <height-DIP> <home|file|navigation|formatting>; set EFRAME_SCREENSHOT_TO".into());
    }
    if std::env::var_os("EFRAME_SCREENSHOT_TO").is_none() {
        return Err("Set EFRAME_SCREENSHOT_TO to an output PNG path".into());
    }
    let width: f32 = args[0].parse()?;
    let height: f32 = args[1].parse()?;
    if !(640.0..=1920.0).contains(&width) || !(480.0..=1080.0).contains(&height) {
        return Err("Capture dimensions outside 640–1920 × 480–1080 DIP".into());
    }
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(if args[2] == "formatting" {
            "examples/Formatting.bfdoc"
        } else {
            "examples/Welcome.bfdoc"
        });
    let session = if args[2] == "file" {
        Session::default()
    } else {
        Session::new(storage::load(&fixture)?, Some(fixture))
    };
    let mode = args[2].clone();
    eframe::run_native(
        "BreakFree Docs capture",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([width, height])
                .with_decorations(false),
            ..Default::default()
        },
        Box::new(move |cc| {
            let mut app = DocsApp::new(session, &cc.egui_ctx);
            match mode.as_str() {
                "file" => app.execute(Action::File, &cc.egui_ctx),
                "navigation" => app.navigation = true,
                _ => {}
            }
            Ok(Box::new(app))
        }),
    )?;
    Ok(())
}
