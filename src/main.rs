fn main() -> eframe::Result {
    eframe::run_native(
        "Grippy",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

#[derive(Default)]
struct App {}
impl App {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Self {}
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| ui.label("Hello, world!"));
    }
}
