//! Get the game → Set up → Play, end to end, with stand-ins for the port's programs: a `dcdata` that prints what the
//! real one prints and makes a data folder, and a `darkcloud` that exits however `DC_TEST_EXIT` says.
#![cfg(unix)]

use chronicle_launcher_core::library::{self, Next};
use chronicle_launcher_core::{paths, play, process, setup};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::Mutex;

fn script(path: &Path, body: &str) {
    std::fs::write(path, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn set_up_then_play() {
    let root = tempfile::tempdir().unwrap();
    let build = tempfile::tempdir().unwrap();
    script(
        &build.path().join("dcdata"),
        r#"[ "$1" = extract ] || exit 2
echo "source: $2 (ISO 9660 image)"
echo "release: PAL prototype (July 12, 2001)"
echo "index: 3 records, 3 files, 30 bytes"
mkdir -p "$3" && echo '{"release":"PAL prototype (July 12, 2001)","languages":[2,3,4,5,6]}' > "$3/languages.json"
echo "verified 3/3 files in $3""#,
    );
    script(
        &build.path().join("darkcloud"),
        r#"[ "$1" = --data ] && [ "$3" = --save ] || exit 2
[ -f "$2/languages.json" ] || exit 3
echo "config: loaded $4/config.json"
echo "mods: ${MOONMOD_NO_MODS:-on}"
exit ${DC_TEST_EXIT:-0}"#,
    );

    let view = library::choose_build(root.path(), "chronicle", build.path()).unwrap();
    assert_eq!(view.next, Next::SetUp);
    let built = view.build.unwrap();
    let dirs = paths::game_dirs(root.path(), "chronicle");

    let child =
        Mutex::new(process::spawn(setup::command(&built, Path::new("/discs/dc.iso"), &dirs)).unwrap());
    let mut stages = Vec::new();
    let ended = process::follow(&child, &dirs.logs.join("setup.log"), |line| {
        stages.extend(setup::Stage::of_line(line));
    })
    .unwrap();
    assert!(ended.success());
    assert_eq!(stages.last(), Some(&setup::Stage::Preparing));
    let done = setup::SetUp {
        disc: "/discs/dc.iso".into(),
        release: Some("PAL prototype (July 12, 2001)".into()),
        build: built.dir.clone(),
        finished: 1,
    };
    setup::finish(&dirs, &done).unwrap();
    assert_eq!(
        library::view_of(root.path(), "chronicle").unwrap().next,
        Next::Play
    );

    let run = |exit: &str| {
        let mut command = play::command(&built, &dirs, false);
        command.env("DC_TEST_EXIT", exit);
        let child = Mutex::new(process::spawn(command).unwrap());
        play::outcome(&process::follow(&child, &dirs.logs.join("play.log"), |_| {}).unwrap())
    };
    assert_eq!(run("0"), play::Outcome::Closed);
    assert!(matches!(run("4"), play::Outcome::Crashed { .. }));
    let log = std::fs::read_to_string(dirs.logs.join("play.log")).unwrap();
    assert!(log.contains(&format!("config: loaded {}/config.json", dirs.save.display())));
    assert!(
        log.contains("mods: 1"),
        "the Mods switch is off by default: {log}"
    );

    let command = play::command(&built, &dirs, true);
    let child = Mutex::new(process::spawn(command).unwrap());
    process::follow(&child, &dirs.logs.join("mods-on.log"), |_| {}).unwrap();
    let log = std::fs::read_to_string(dirs.logs.join("mods-on.log")).unwrap();
    assert!(
        log.contains("mods: on"),
        "with the switch on, the variable is not set: {log}"
    );

    std::fs::remove_file(dirs.data.join("languages.json")).unwrap();
    assert_eq!(run("0"), play::Outcome::NoData);
}
