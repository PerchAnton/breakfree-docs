//! Development-only native screenshot harness; never part of the editor CLI.
use breakfree_docs::{
    gui::{Action, DocsApp},
    session::Session,
    storage,
};
use eframe::egui;

struct Capture {
    app: DocsApp,
    path: String,
    frames: usize,
}

impl eframe::App for Capture {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.app.update_ui(ctx);
        for event in ctx.input(|i| i.events.clone()) {
            if let egui::Event::Screenshot { image, .. } = event {
                let rgba: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                image::save_buffer(
                    &self.path,
                    &rgba,
                    image.width() as u32,
                    image.height() as u32,
                    image::ColorType::Rgba8,
                )
                .expect("Save native capture");
                // Exit only this development harness; the editor correctly blocks
                // ordinary window-close requests while its Font draft is open.
                std::process::exit(0);
            }
        }
        // Native areas have initial measurement passes; capture a settled frame.
        self.frames += 1;
        if self.frames == 8 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
        }
        if self.frames <= 8 {
            ctx.request_repaint();
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 3
        || ![
            "home",
            "file",
            "navigation",
            "formatting",
            "font",
            "collapsed",
            "temporary",
        ]
        .contains(&args[2].as_str())
    {
        return Err("Usage: capture_ui <width-DIP> <height-DIP> <home|file|navigation|formatting|font|collapsed|temporary>; set BREAKFREE_CAPTURE_TO".into());
    }
    let path = std::env::var("BREAKFREE_CAPTURE_TO")
        .map_err(|_| "Set BREAKFREE_CAPTURE_TO to an output PNG path")?;
    if std::env::var_os("EFRAME_SCREENSHOT_TO").is_some() {
        return Err("Unset EFRAME_SCREENSHOT_TO; it captures before modal layout settles".into());
    }
    let width: f32 = args[0].parse()?;
    let height: f32 = args[1].parse()?;
    if !(640.0..=1920.0).contains(&width) || !(480.0..=1080.0).contains(&height) {
        return Err("Capture dimensions outside 640–1920 × 480–1080 DIP".into());
    }
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        if ["formatting", "font"].contains(&args[2].as_str()) {
            "examples/Formatting.bfdoc"
        } else {
            "examples/Welcome.bfdoc"
        },
    );
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
                "collapsed" => app.ribbon_collapsed = true,
                "temporary" => {
                    app.ribbon_collapsed = true;
                    app.ribbon_temporary = true;
                }
                "font" => {
                    app.execute(Action::SelectAll, &cc.egui_ctx);
                    app.execute(Action::FontDialog, &cc.egui_ctx);
                }
                _ => {}
            }
            Ok(Box::new(Capture {
                app,
                path,
                frames: 0,
            }))
        }),
    )?;
    Ok(())
}
