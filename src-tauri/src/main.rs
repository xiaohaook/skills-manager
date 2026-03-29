#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bin_install;
mod claws;
mod config;
mod file_ops;
mod hot_skills;
mod install;
mod models;
mod scan;
mod skill_md;
mod sources;
mod symlink_index;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            file_ops::check_path_exists,
            file_ops::is_symlink,
            scan::scan_skills,
            scan::scan_remote_claw,
            file_ops::copy_skill,
            claws::get_local_claws,
            file_ops::show_in_finder,
            file_ops::delete_skill,
            file_ops::open_file_in_editor,
            install::install_from_github,
            hot_skills::get_hot_skills,
            hot_skills::clear_hot_skills_cache,
            sources::add_custom_source,
            sources::remove_custom_source,
            sources::get_custom_sources,
            sources::set_github_token,
            bin_install::install_bins_with_homebrew,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
