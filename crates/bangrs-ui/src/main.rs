use bangrs_core::ViewModel;

slint::include_modules!();

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let _vm = ViewModel::default();
    todo!("green: wire up PlayerHandle, scanner worker thread, Slint callbacks, 50ms bind timer, event loop")
}
