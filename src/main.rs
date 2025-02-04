mod app;
mod storage;
mod commands;

fn main() {
    let options = eframe::NativeOptions::default();
    if let Err(e) = eframe::run_native(
        "Build Tool GUI",
        options,
        Box::new(|_cc| Ok(Box::new(app::BuildApp::default()))),
    ) {
        eprintln!("Error running the application: {}", e);
    }
}
