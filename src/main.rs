use breakfree_docs::{
    VERSION,
    integration::{self, Launch},
    session::Session,
};
fn main() {
    let launch = match integration::arguments(std::env::args_os().skip(1)) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("BreakFree Docs: {e}");
            std::process::exit(2)
        }
    };
    let session = match launch {
        Launch::Capabilities => {
            println!("{}", integration::capabilities());
            return;
        }
        Launch::Version => {
            println!("BreakFree Docs {VERSION}");
            return;
        }
        Launch::Help => {
            println!(
                "BreakFree Docs\nUsage: breakfree-docs [--new | --open <absolute_path> | --capabilities | --version]"
            );
            return;
        }
        Launch::New => Session::default(),
        Launch::Open(path) => match Session::open(&path) {
            Ok(session) => session,
            Err(e) => {
                eprintln!("BreakFree Docs: {e}");
                std::process::exit(1)
            }
        },
    };
    #[cfg(feature = "desktop")]
    if let Err(e) = breakfree_docs::gui::run(session) {
        eprintln!("BreakFree Docs: {e}");
        std::process::exit(1);
    }
    #[cfg(not(feature = "desktop"))]
    {
        let _ = session;
        eprintln!("Desktop support was not compiled. Rebuild with the desktop feature.");
        std::process::exit(1);
    }
}
