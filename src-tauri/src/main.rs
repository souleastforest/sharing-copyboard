#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use sharing_copyboard::run;

fn main() {
    run();
}
