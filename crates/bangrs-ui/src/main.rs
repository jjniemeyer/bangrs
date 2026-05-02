use bangrs_core::ViewModel;
use slint::ComponentHandle;
use bangrs_scan::{FilesystemScanner, Scanner};
use bangrs_ui::dispatcher::{Dispatcher, SlintDispatcher};
use bangrs_ui::handlers::{ActivationHandler, SelectionHandler};
use bangrs_ui::picker::{FolderPicker, RfdPicker};
use bangrs_ui::side_effects::{EngineLink, MainSideEffects, UiSideEffects};
use bangrs_ui::{bind, on_pick_library, MainWindow, PickState};
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let window = MainWindow::new()?;

    let vm: Arc<Mutex<ViewModel>> = Arc::new(Mutex::new(ViewModel::default()));
    let link: Arc<Mutex<Option<EngineLink>>> = Arc::new(Mutex::new(None));
    let handlers: Arc<Mutex<Option<(SelectionHandler, ActivationHandler)>>> =
        Arc::new(Mutex::new(None));
    let selected_row: Arc<Mutex<i32>> = Arc::new(Mutex::new(-1));

    let picker: Arc<dyn FolderPicker> = Arc::new(RfdPicker);
    let scanner: Arc<dyn Scanner + Send + Sync> = Arc::new(FilesystemScanner);
    let dispatcher: Arc<dyn Dispatcher> = Arc::new(SlintDispatcher);
    let pick_state = Arc::new(PickState::new());
    let side_effects: Arc<dyn UiSideEffects> = Arc::new(MainSideEffects {
        link: link.clone(),
        vm: vm.clone(),
        handlers: handlers.clone(),
        window_weak: window.as_weak(),
    });

    {
        let picker = picker.clone();
        let scanner = scanner.clone();
        let dispatcher = dispatcher.clone();
        let pick_state = pick_state.clone();
        let side_effects = side_effects.clone();
        window.on_pick_library(move || {
            on_pick_library(
                picker.clone(),
                scanner.clone(),
                dispatcher.clone(),
                pick_state.clone(),
                side_effects.clone(),
            );
        });
    }

    {
        let link = link.clone();
        let selected_row = selected_row.clone();
        window.on_play_clicked(move || {
            if let Some(l) = link.lock().unwrap().as_ref() {
                let row = *selected_row.lock().unwrap();
                if row >= 0
                    && let Some(id) = l.track_ids.get(row as usize).copied()
                {
                    let _ = l.handle.play(id);
                    return;
                }
                let _ = l.handle.resume();
            }
        });
    }

    {
        let link = link.clone();
        window.on_pause_clicked(move || {
            if let Some(l) = link.lock().unwrap().as_ref() {
                let _ = l.handle.pause();
            }
        });
    }

    {
        let link = link.clone();
        window.on_stop_clicked(move || {
            if let Some(l) = link.lock().unwrap().as_ref() {
                let _ = l.handle.stop();
            }
        });
    }

    {
        let selected_row = selected_row.clone();
        let handlers = handlers.clone();
        window.on_selection_changed(move |idx| {
            *selected_row.lock().unwrap() = idx;
            if let Some((sel, _)) = handlers.lock().unwrap().as_ref() {
                sel(idx);
            }
        });
    }

    {
        let handlers = handlers.clone();
        window.on_track_activated(move |idx| {
            if let Some((_, act)) = handlers.lock().unwrap().as_ref() {
                act(idx);
            }
        });
    }

    let timer = slint::Timer::default();
    {
        let link = link.clone();
        let vm = vm.clone();
        let window_weak = window.as_weak();
        timer.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(50),
            move || {
                let Some(w) = window_weak.upgrade() else {
                    return;
                };
                let mut updated = vm.lock().unwrap().clone();
                let mut saw_event = false;
                if let Some(l) = link.lock().unwrap().as_ref() {
                    while let Ok(ev) = l.ev_rx.try_recv() {
                        updated = updated.apply(&ev);
                        saw_event = true;
                    }
                    let pos_ms = l.handle.position().as_millis() as u64;
                    if pos_ms != updated.position_ms {
                        updated.position_ms = pos_ms;
                        saw_event = true;
                    }
                }
                if saw_event {
                    *vm.lock().unwrap() = updated;
                    bind(&w, &vm.lock().unwrap());
                }
            },
        );
    }

    bind(&window, &vm.lock().unwrap());
    window.run()?;

    if let Some(old) = link.lock().unwrap().take() {
        let _ = old.handle.shutdown();
    }
    Ok(())
}
