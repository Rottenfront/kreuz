use kreuz_ui::DummyView;
use kreuz_vello::make_vello_app;
use kreuz_winit::run_with_winit;

fn main() {
    let _ = run_with_winit(make_vello_app(DummyView));
}
