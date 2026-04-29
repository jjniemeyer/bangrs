use bangrs_audio::{AudioEngine, CpalEngine, Event};
use bangrs_core::{Library, TrackId, ViewModel};
use bangrs_scan::{FilesystemScanner, Scanner};
use bangrs_ui::handlers::{wire_handlers, ActivationHandler, CommandSink, SelectionHandler};
use bangrs_ui::PlayerHandle;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::AtomicU64;
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::Duration;

slint::include_modules!();

struct EngineLink {
    handle: PlayerHandle,
    ev_rx: crossbeam_channel::Receiver<Event>,
    track_ids: Vec<TrackId>,
}

fn spawn_engine(library: Arc<Library>, track_ids: Vec<TrackId>) -> EngineLink {
    let (cmd_tx, cmd_rx) = mpsc::sync_channel(32);
    let (ev_tx, ev_rx) = crossbeam_channel::bounded(64);
    let position = Arc::new(AtomicU64::new(0));

    let pos_audio = position.clone();
    let lib_audio = library.clone();
    thread::spawn(move || {
        CpalEngine.run(lib_audio, cmd_rx, ev_tx, pos_audio);
    });

    let handle = PlayerHandle::new(cmd_tx, position);
    EngineLink { handle, ev_rx, track_ids }
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let window = MainWindow::new()?;
    let vm = Rc::new(RefCell::new(ViewModel::default()));
    let link: Rc<RefCell<Option<EngineLink>>> = Rc::new(RefCell::new(None));
    let selected_row: Rc<RefCell<i32>> = Rc::new(RefCell::new(-1));
    let handlers: Rc<RefCell<Option<(SelectionHandler, ActivationHandler)>>> =
        Rc::new(RefCell::new(None));

    {
        let link = link.clone();
        let vm = vm.clone();
        let handlers = handlers.clone();
        let window_weak = window.as_weak();
        window.on_pick_library(move || {
            let Some(path) = rfd::FileDialog::new().pick_folder() else {
                return;
            };
            let scanner = FilesystemScanner;
            let tracks: Vec<_> = scanner.scan(&path).filter_map(Result::ok).collect();
            let track_ids: Vec<TrackId> = tracks.iter().map(|t| t.id).collect();
            let library = Arc::new(Library::new(tracks));

            if let Some(old) = link.borrow_mut().take() {
                let _ = old.handle.shutdown();
            }
            let new_link = spawn_engine(library.clone(), track_ids.clone());
            let sink: Arc<dyn CommandSink> = Arc::new(new_link.handle.clone());
            *handlers.borrow_mut() = Some(wire_handlers(sink, Arc::new(track_ids)));
            *link.borrow_mut() = Some(new_link);

            let updated = vm.borrow().clone().set_library(library);
            *vm.borrow_mut() = updated;

            if let Some(w) = window_weak.upgrade() {
                bind(&w, &vm.borrow());
            }
        });
    }

    {
        let link = link.clone();
        let selected_row = selected_row.clone();
        window.on_play_clicked(move || {
            if let Some(l) = link.borrow().as_ref() {
                let row = *selected_row.borrow();
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
            if let Some(l) = link.borrow().as_ref() {
                let _ = l.handle.pause();
            }
        });
    }

    {
        let link = link.clone();
        window.on_stop_clicked(move || {
            if let Some(l) = link.borrow().as_ref() {
                let _ = l.handle.stop();
            }
        });
    }

    {
        let selected_row = selected_row.clone();
        let handlers = handlers.clone();
        window.on_selection_changed(move |idx| {
            *selected_row.borrow_mut() = idx;
            if let Some((sel, _)) = handlers.borrow().as_ref() {
                sel(idx);
            }
        });
    }

    {
        let handlers = handlers.clone();
        window.on_track_activated(move |idx| {
            if let Some((_, act)) = handlers.borrow().as_ref() {
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
                let mut updated = vm.borrow().clone();
                let mut saw_event = false;
                if let Some(l) = link.borrow().as_ref() {
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
                    *vm.borrow_mut() = updated;
                    bind(&w, &vm.borrow());
                }
            },
        );
    }

    bind(&window, &vm.borrow());
    window.run()?;

    if let Some(old) = link.borrow_mut().take() {
        let _ = old.handle.shutdown();
    }
    Ok(())
}

fn bind(window: &MainWindow, vm: &ViewModel) {
    window.set_is_playing(vm.is_playing);
    window.set_is_paused(vm.is_paused);
    window.set_current_track_id(vm.current_track.map(|TrackId(id)| id as i32).unwrap_or(-1));
    window.set_position_ms(vm.position_ms as i32);
    window.set_error_banner(vm.error_banner.clone().unwrap_or_default().into());
    let rows: Vec<slint::StandardListViewItem> = vm
        .tracks
        .iter()
        .map(|t| {
            let label = if t.artist.is_empty() {
                t.title.clone()
            } else {
                format!("{} — {}", t.artist, t.title)
            };
            slint::StandardListViewItem::from(slint::SharedString::from(label))
        })
        .collect();
    window.set_tracks(slint::ModelRc::new(slint::VecModel::from(rows)));
}
