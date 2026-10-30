//! The launcher's commands. Each is a thin call into `chronicle-launcher-core`; set-up and play run on threads of
//! their own and tell the page how they go with events:
//!
//! - `activity` (a `GameView`): a game started or stopped setting up or playing, or its set-up moved on;
//! - `setup-line` (`{ id, line }`): a line the extractor printed;
//! - `setup-finished` (`{ id, ok, message }`);
//! - `game-ended` (`{ id, outcome }`).

use chronicle_launcher_core::library::{self, Activity, GameView};
use chronicle_launcher_core::{discs, games, mods, paths, play, process, saves, scene, setup};
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager, State};

/// A mod unpacked and checked, waiting for the player to install it from its preview.
#[derive(Default)]
struct Staged(Mutex<HashMap<String, mods::Staged>>);

/// What runs for each game: its process, and what it is doing.
#[derive(Default)]
struct Runs(Mutex<HashMap<String, Run>>);

struct Run {
    child: Arc<Mutex<std::process::Child>>,
    activity: Activity,
}

impl Runs {
    fn activity(&self, id: &str) -> Activity {
        self.0
            .lock()
            .unwrap()
            .get(id)
            .map(|r| r.activity.clone())
            .unwrap_or_default()
    }

    fn busy(&self, id: &str) -> Result<(), String> {
        match self.activity(id) {
            Activity::Idle => Ok(()),
            Activity::Playing => Err("The game is running. Close it first.".into()),
            Activity::SettingUp { .. } => Err("The game is being set up. Wait for it, or cancel it.".into()),
            Activity::Rendering => {
                Err("The launcher is taking a picture of the game. It takes a moment.".into())
            }
        }
    }
}

fn root() -> PathBuf {
    paths::root()
}

fn game(id: &str) -> Result<&'static games::Game, String> {
    games::find(id).ok_or_else(|| format!("No game called {id}"))
}

fn view(runs: &Runs, id: &str) -> Result<GameView, String> {
    let mut view = library::view_of(&root(), id)?;
    view.activity = runs.activity(id);
    Ok(view)
}

fn announce(app: &AppHandle, id: &str) {
    if let Ok(view) = view(&app.state::<Runs>(), id) {
        let _ = app.emit("activity", view);
    }
}

#[tauri::command]
fn library(runs: State<Runs>) -> Result<Vec<GameView>, String> {
    games::all().iter().map(|g| view(&runs, &g.id)).collect()
}

#[tauri::command]
fn choose_build(runs: State<Runs>, id: String, folder: PathBuf) -> Result<GameView, String> {
    runs.busy(&id)?;
    library::choose_build(&root(), &id, &folder)?;
    view(&runs, &id)
}

#[tauri::command]
fn forget_build(runs: State<Runs>, id: String) -> Result<GameView, String> {
    runs.busy(&id)?;
    library::forget_build(&root(), &id)?;
    view(&runs, &id)
}

#[tauri::command]
fn scan_discs(id: String, folder: PathBuf) -> Result<discs::Scan, String> {
    let game = game(&id)?;
    library::remember_disc_folder(&root(), &id, &folder)?;
    Ok(discs::scan(game, &folder))
}

#[derive(Clone, Serialize)]
struct Line<'a> {
    id: &'a str,
    line: &'a str,
}

#[derive(Clone, Serialize)]
struct Finished {
    id: String,
    ok: bool,
    message: String,
}

#[tauri::command]
fn set_up(app: AppHandle, runs: State<Runs>, id: String, disc: PathBuf) -> Result<GameView, String> {
    runs.busy(&id)?;
    let game = game(&id)?;
    let current = library::view_of(&root(), &id)?;
    let build = current.build.ok_or("Choose a build first.")?;
    let found = discs::identify(game, &disc)?;
    let dirs = paths::game_dirs(&root(), &id);
    let child = process::spawn(setup::command(&build, &disc, &dirs))
        .map_err(|e| format!("{} could not start: {e}", build.extractor.display()))?;
    let child = Arc::new(Mutex::new(child));
    let starting = Activity::SettingUp(setup::Progress::starting());
    runs.0.lock().unwrap().insert(
        id.clone(),
        Run {
            child: child.clone(),
            activity: starting,
        },
    );

    let log = dirs.logs.join(format!("setup-{}.log", saves::now()));
    std::thread::spawn(move || {
        let runs = app.state::<Runs>();
        let mut last = String::new();
        let mut release = found.release;
        let ended = process::follow(&child, &log, |line| {
            if let Some(named) = setup::release_in_line(line) {
                release = Some(named.to_string());
            }
            let _ = app.emit("setup-line", Line { id: &id, line });
            if !line.trim().is_empty() {
                last = line.to_string();
            }
            if let Some(progress) = setup::progress_of_line(game.progress, line) {
                if let Some(run) = runs.0.lock().unwrap().get_mut(&id) {
                    run.activity = Activity::SettingUp(progress);
                }
                announce(&app, &id);
            }
        });
        let finished = match ended {
            Ok(ended) if ended.success() => {
                let done = setup::SetUp {
                    disc,
                    release,
                    build: build.dir,
                    finished: saves::now(),
                };
                match setup::finish(&dirs, &done) {
                    Ok(()) => Finished {
                        id: id.clone(),
                        ok: true,
                        message: "Set up. Ready to play.".into(),
                    },
                    Err(e) => Finished {
                        id: id.clone(),
                        ok: false,
                        message: format!("Could not record the set-up: {e}"),
                    },
                }
            }
            Ok(ended) if ended.code.is_none() => Finished {
                id: id.clone(),
                ok: false,
                message: "Set-up was stopped.".into(),
            },
            Ok(_) => Finished {
                id: id.clone(),
                ok: false,
                message: format!("Set-up failed: {last}"),
            },
            Err(e) => Finished {
                id: id.clone(),
                ok: false,
                message: format!("Set-up failed: {e}"),
            },
        };
        let set_up = finished.ok;
        runs.0.lock().unwrap().remove(&id);
        let _ = app.emit("setup-finished", finished);
        announce(&app, &id);
        // The first backdrop: a frame of the game the player just set up.
        if set_up {
            let _ = start_scene(&app, &id, Render::Now);
        }
    });
    view(&runs, &current.game.id)
}

#[derive(Clone, Serialize)]
struct SceneReady {
    id: String,
    ok: bool,
    message: String,
    /// A render the player did not ask for (after a session, the town changed): the page updates without a word.
    quiet: bool,
}

/// Why a backdrop is rendered.
#[derive(Clone, Copy, PartialEq)]
enum Render {
    /// Set-up just finished, or the player asked for a new one.
    Now,
    /// A session ended: only if the latest save is somewhere else, and never over the player's own picture.
    IfMoved,
}

/// Renders the page's backdrop with the port, on a thread: the town of the player's latest save, or the default
/// place when there is no save or the port cannot start there. `scene-ready` says how it went.
fn start_scene(app: &AppHandle, id: &str, why: Render) -> Result<(), String> {
    let runs = app.state::<Runs>();
    runs.busy(id)?;
    let game = game(id)?;
    let current = library::view_of(&root(), id)?;
    let build = current.build.ok_or("Choose a build first.")?;
    if current.set_up.is_none() {
        return Err("Set the game up from your disc first.".into());
    }
    let dirs = paths::game_dirs(&root(), id);
    let place = scene::place(game, &saves::list(game.saves, &dirs.save));
    let default = scene::default_place(game).map(str::to_string);
    if why == Render::IfMoved {
        let shown = scene::shown(&dirs);
        if shown.own || place.is_none() || shown.place == place {
            return Ok(());
        }
    }
    let out = scene::pending(&dirs);
    let _ = std::fs::remove_file(&out);
    let child = process::spawn(scene::command(game, &build, &dirs, place.as_deref(), &out))
        .map_err(|e| format!("{} could not start: {e}", build.executable.display()))?;
    let child = Arc::new(Mutex::new(child));
    runs.0.lock().unwrap().insert(
        id.to_string(),
        Run {
            child: child.clone(),
            activity: Activity::Rendering,
        },
    );
    announce(app, id);
    let (app, id) = (app.clone(), id.to_string());
    std::thread::spawn(move || {
        let follow = |child: &Arc<Mutex<_>>| {
            let log = dirs.logs.join(format!("scene-{}.log", saves::now()));
            match process::follow(child, &log, |_| {}) {
                Ok(ended) if ended.success() => scene::adopt(&dirs, &out),
                Ok(_) => Err("The game did not finish its picture; the log says why.".to_string()),
                Err(e) => Err(e.to_string()),
            }
        };
        let mut rendered = place.clone().or(default.clone());
        let mut result = follow(&child);
        // A save on a map the port cannot start on: the default place instead.
        if result.is_err() && place.is_some() && place != default {
            let retry = process::spawn(scene::command(game, &build, &dirs, None, &out))
                .map(|c| Arc::new(Mutex::new(c)));
            if let Ok(child) = retry {
                if let Some(run) = app.state::<Runs>().0.lock().unwrap().get_mut(&id) {
                    run.child = child.clone();
                }
                rendered = default.clone();
                result = follow(&child);
            }
        }
        if result.is_ok() {
            let _ = scene::record(
                &dirs,
                &scene::Shown {
                    place: rendered,
                    own: false,
                },
            );
        }
        let _ = std::fs::remove_file(&out);
        app.state::<Runs>().0.lock().unwrap().remove(&id);
        let (ok, message) = match result {
            Ok(()) => (true, "New background taken.".to_string()),
            Err(e) => (false, e),
        };
        let quiet = why == Render::IfMoved;
        let _ = app.emit(
            "scene-ready",
            SceneReady {
                id: id.clone(),
                ok,
                message,
                quiet,
            },
        );
        announce(&app, &id);
    });
    Ok(())
}

/// The page's backdrop as a `data:` URL, or nothing yet.
#[tauri::command]
fn scene_image(id: String) -> Result<Option<String>, String> {
    game(&id)?;
    Ok(scene::data_url(&paths::game_dirs(&root(), &id)))
}

#[tauri::command]
fn render_scene(app: AppHandle, id: String) -> Result<(), String> {
    start_scene(&app, &id, Render::Now)
}

/// Makes the player's own picture the backdrop.
#[tauri::command]
fn use_scene(id: String, picture: PathBuf) -> Result<(), String> {
    game(&id)?;
    let dirs = paths::game_dirs(&root(), &id);
    scene::adopt(&dirs, &picture)?;
    scene::record(
        &dirs,
        &scene::Shown {
            place: None,
            own: true,
        },
    )
    .map_err(|e| e.to_string())
}

#[tauri::command]
fn cancel_set_up(runs: State<Runs>, id: String) {
    if let Some(run) = runs.0.lock().unwrap().get(&id) {
        if matches!(run.activity, Activity::SettingUp { .. }) {
            process::kill(&run.child);
        }
    }
}

#[tauri::command]
fn remove_set_up(runs: State<Runs>, id: String) -> Result<GameView, String> {
    runs.busy(&id)?;
    setup::remove(&paths::game_dirs(&root(), &id)).map_err(|e| e.to_string())?;
    view(&runs, &id)
}

#[derive(Clone, Serialize)]
struct GameEnded {
    id: String,
    outcome: play::Outcome,
}

#[tauri::command]
fn play(app: AppHandle, runs: State<Runs>, id: String) -> Result<GameView, String> {
    runs.busy(&id)?;
    let current = library::view_of(&root(), &id)?;
    let build = current.build.ok_or("Choose a build first.")?;
    if current.set_up.is_none() {
        return Err("Set the game up from your disc first.".into());
    }
    let dirs = paths::game_dirs(&root(), &id);
    std::fs::create_dir_all(&dirs.save).map_err(|e| e.to_string())?;
    let child = process::spawn(play::command(&build, &dirs, current.mods_on))
        .map_err(|e| format!("{} could not start: {e}", build.executable.display()))?;
    let child = Arc::new(Mutex::new(child));
    runs.0.lock().unwrap().insert(
        id.clone(),
        Run {
            child: child.clone(),
            activity: Activity::Playing,
        },
    );

    let started = saves::now();
    let log_name = format!("play-{started}.log");
    let log = dirs.logs.join(&log_name);
    let thread_id = id.clone();
    std::thread::spawn(move || {
        let ended = process::follow(&child, &log, |_| {});
        let outcome = match ended {
            Ok(ended) => {
                let outcome = play::outcome(&ended);
                match &outcome {
                    play::Outcome::Closed => play::clear_crash(&dirs.logs),
                    play::Outcome::NoData => setup::forget(&dirs),
                    play::Outcome::Crashed { what } => {
                        let crash = play::Crash {
                            what: what.clone(),
                            when: saves::now(),
                            log: log_name,
                            ended_code: ended.code,
                            ended_signal: ended.signal,
                        };
                        let _ = play::record_crash(&dirs.logs, &crash);
                    }
                }
                outcome
            }
            Err(e) => play::Outcome::Crashed {
                what: format!("The launcher lost track of it ({e})"),
            },
        };
        app.state::<Runs>().0.lock().unwrap().remove(&thread_id);
        let _ = app.emit(
            "game-ended",
            GameEnded {
                id: thread_id.clone(),
                outcome,
            },
        );
        announce(&app, &thread_id);
        // The page follows the player: a new backdrop when they saved in another town.
        let _ = start_scene(&app, &thread_id, Render::IfMoved);
    });
    view(&runs, &id)
}

#[derive(Serialize)]
struct SavesView {
    saves: Vec<saves::Save>,
    backups: Vec<saves::Backup>,
}

#[tauri::command]
fn saves(id: String) -> Result<SavesView, String> {
    let format = game(&id)?.saves;
    let dirs = paths::game_dirs(&root(), &id);
    Ok(SavesView {
        saves: saves::list(format, &dirs.save),
        backups: saves::backups(format, &dirs.root),
    })
}

#[tauri::command]
fn back_up_saves(runs: State<Runs>, id: String) -> Result<Option<saves::Backup>, String> {
    if matches!(runs.activity(&id), Activity::Playing) {
        return Err("The game is running. Close it first, so its saves are not half written.".into());
    }
    let dirs = paths::game_dirs(&root(), &id);
    saves::back_up(game(&id)?.saves, &dirs.root, &dirs.save).map_err(|e| e.to_string())
}

#[tauri::command]
fn restore_saves(runs: State<Runs>, id: String, backup: String) -> Result<(), String> {
    runs.busy(&id)?;
    let dirs = paths::game_dirs(&root(), &id);
    saves::restore(game(&id)?.saves, &dirs.root, &dirs.save, &backup)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

fn report_text(id: &str) -> Result<String, String> {
    let game = game(id)?;
    let dirs = paths::game_dirs(&root(), id);
    let crash = play::last_crash(&dirs.logs).ok_or("No crash to report.")?;
    let log = std::fs::read_to_string(dirs.logs.join(&crash.log)).unwrap_or_default();
    let build = library::view_of(&root(), id)?
        .build
        .map(|b| b.dir)
        .unwrap_or_default();
    let ended = process::Ended {
        code: crash.ended_code,
        signal: crash.ended_signal,
    };
    let mut report = play::report(game, &build, &ended, &log, dirs_home().as_deref());
    if let Some(own) = play::game_crash_report(&dirs.save, crash.when) {
        report.push_str(&format!(
            "\nThe game's own crash report:\n```\n{}\n```\n",
            own.trim_end()
        ));
    }
    Ok(report)
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).map(PathBuf::from)
}

#[tauri::command]
fn crash_report(id: String) -> Result<String, String> {
    report_text(&id)
}

/// Writes the report to the Downloads folder (or the game's logs folder without one) and says where.
#[tauri::command]
fn save_report(id: String) -> Result<String, String> {
    let text = report_text(&id)?;
    let dirs = paths::game_dirs(&root(), &id);
    let folder = dirs_downloads().unwrap_or(dirs.logs);
    let file = folder.join(format!("{id}-report-{}.txt", saves::now()));
    std::fs::write(&file, text).map_err(|e| e.to_string())?;
    Ok(file.display().to_string())
}

fn dirs_downloads() -> Option<PathBuf> {
    let home = dirs_home()?;
    let downloads = home.join("Downloads");
    downloads.is_dir().then_some(downloads)
}

#[tauri::command]
fn open_folder(app: AppHandle, id: String, which: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    let dirs = paths::game_dirs(&root(), &id);
    let mods_dir = library::view_of(&root(), &id)?.mods_dir;
    let folder: &Path = match which.as_str() {
        "saves" => &dirs.save,
        "logs" => &dirs.logs,
        "game" => &dirs.root,
        "mods" => &mods_dir,
        _ => return Err(format!("No folder called {which}")),
    };
    std::fs::create_dir_all(folder).map_err(|e| e.to_string())?;
    app.opener()
        .open_path(folder.display().to_string(), None::<&str>)
        .map_err(|e| e.to_string())
}

/// The game's mods, as the launcher and the game both read them.
#[derive(Serialize)]
struct ModsView {
    mods: Vec<mods::Mod>,
}

fn mods_place(id: &str) -> Result<(PathBuf, PathBuf), String> {
    let view = library::view_of(&root(), id)?;
    Ok((view.mods_dir, view.save))
}

/// The game writes the same profile; changing it while the game runs could lose one side's change.
fn not_playing(runs: &Runs, id: &str) -> Result<(), String> {
    match runs.activity(id) {
        Activity::Playing => Err("Close the game first: it is using the mod settings.".into()),
        _ => Ok(()),
    }
}

#[tauri::command]
fn mods_list(id: String) -> Result<ModsView, String> {
    let (dir, save) = mods_place(&id)?;
    Ok(ModsView {
        mods: mods::list(&dir, &save, &id)?,
    })
}

#[tauri::command]
fn set_mod_enabled(runs: State<Runs>, id: String, mod_id: String, on: bool) -> Result<(), String> {
    not_playing(&runs, &id)?;
    let (dir, save) = mods_place(&id)?;
    mods::set_enabled(&dir, &save, &id, &mod_id, on)
}

#[tauri::command]
fn set_mod_setting(
    runs: State<Runs>,
    id: String,
    mod_id: String,
    setting: String,
    value: Option<serde_json::Value>,
) -> Result<(), String> {
    not_playing(&runs, &id)?;
    let (dir, save) = mods_place(&id)?;
    mods::set_setting(&dir, &save, &id, &mod_id, &setting, value)
}

#[tauri::command]
fn set_mods_on(runs: State<Runs>, id: String, on: bool) -> Result<GameView, String> {
    not_playing(&runs, &id)?;
    library::set_mods_on(&root(), &id, on)?;
    view(&runs, &id)
}

/// Unpacks or copies a `.zip` or folder and checks it; nothing is installed until `install_mod`.
#[tauri::command]
fn preview_mod(staged: State<Staged>, id: String, source: PathBuf) -> Result<mods::Preview, String> {
    let (dir, _) = mods_place(&id)?;
    if let Some(old) = staged.0.lock().unwrap().remove(&id) {
        old.discard();
    }
    let package = mods::stage(&source, &dir, &id)?;
    let preview = package.preview.clone();
    staged.0.lock().unwrap().insert(id, package);
    Ok(preview)
}

#[tauri::command]
fn install_mod(runs: State<Runs>, staged: State<Staged>, id: String) -> Result<String, String> {
    not_playing(&runs, &id)?;
    let (dir, _) = mods_place(&id)?;
    let package = staged
        .0
        .lock()
        .unwrap()
        .remove(&id)
        .ok_or("Nothing waiting to be installed.")?;
    package.install(&dir)
}

#[tauri::command]
fn discard_mod(staged: State<Staged>, id: String) {
    if let Some(package) = staged.0.lock().unwrap().remove(&id) {
        package.discard();
    }
}

#[tauri::command]
fn remove_mod(runs: State<Runs>, id: String, folder: String) -> Result<(), String> {
    not_playing(&runs, &id)?;
    let (dir, _) = mods_place(&id)?;
    mods::remove(&dir, &folder)
}

pub fn run() {
    tauri::Builder::default()
        .manage(Runs::default())
        .manage(Staged::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            library,
            choose_build,
            forget_build,
            scan_discs,
            set_up,
            cancel_set_up,
            remove_set_up,
            play,
            saves,
            back_up_saves,
            restore_saves,
            crash_report,
            save_report,
            open_folder,
            mods_list,
            set_mod_enabled,
            set_mod_setting,
            set_mods_on,
            preview_mod,
            install_mod,
            discard_mod,
            remove_mod,
            scene_image,
            render_scene,
            use_scene,
        ])
        .run(tauri::generate_context!())
        .expect("the launcher's window could not start");
}
