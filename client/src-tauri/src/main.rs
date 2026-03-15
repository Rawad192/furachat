// Point d'entrée Tauri pour le client desktop FuraChat
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("Erreur lors du lancement de l'application Tauri");
}
