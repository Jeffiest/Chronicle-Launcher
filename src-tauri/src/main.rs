// No console window beside the launcher on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    chronicle_launcher_lib::run()
}
