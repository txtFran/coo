use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_autostart::ManagerExt;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Reminder {
    id: u64,
    text: String,
    completed: bool,
}

struct AppState {
    reminders: Mutex<Vec<Reminder>>,
    file_path: PathBuf,
}

// --------------------------------------------------
// Loading reminders
// --------------------------------------------------

fn load_reminders(file_path: &PathBuf) -> Vec<Reminder> {
    if !file_path.exists() {
        return Vec::new();
    }

    match fs::read_to_string(file_path) {
        Ok(contents) => match serde_json::from_str(&contents) {
            Ok(reminders) => reminders,

            Err(error) => {
                eprintln!("Failed to parse reminders: {}", error);
                Vec::new()
            }
        },

        Err(error) => {
            eprintln!("Failed to read reminders: {}", error);
            Vec::new()
        }
    }
}

// --------------------------------------------------
// Saving reminders
// --------------------------------------------------

fn save_reminders(
    file_path: &PathBuf,
    reminders: &[Reminder],
) -> Result<(), String> {
    let json = serde_json::to_string_pretty(reminders)
        .map_err(|error| error.to_string())?;

    fs::write(file_path, json)
        .map_err(|error| error.to_string())?;

    Ok(())
}

// --------------------------------------------------
// Get reminders
// --------------------------------------------------

#[tauri::command]
fn get_reminders(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<Reminder>, String> {
    let reminders = state
        .reminders
        .lock()
        .map_err(|_| "Failed to access reminders".to_string())?;

    Ok(reminders.clone())
}

// --------------------------------------------------
// Add reminder
// --------------------------------------------------

#[tauri::command]
fn add_reminder(
    text: String,
    state: tauri::State<'_, AppState>,
) -> Result<Reminder, String> {
    let mut reminders = state
        .reminders
        .lock()
        .map_err(|_| "Failed to access reminders".to_string())?;

    let next_id = reminders
        .iter()
        .map(|reminder| reminder.id)
        .max()
        .unwrap_or(0)
        + 1;

    let reminder = Reminder {
        id: next_id,
        text,
        completed: false,
    };

    reminders.push(reminder.clone());

    save_reminders(&state.file_path, &reminders)?;

    Ok(reminder)
}

// --------------------------------------------------
// Toggle reminder
// --------------------------------------------------

#[tauri::command]
fn toggle_reminder(
    id: u64,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut reminders = state
        .reminders
        .lock()
        .map_err(|_| "Failed to access reminders".to_string())?;

    let reminder = reminders
        .iter_mut()
        .find(|reminder| reminder.id == id);

    match reminder {
        Some(reminder) => {
            reminder.completed = !reminder.completed;
        }

        None => {
            return Err("Reminder not found".to_string());
        }
    }

    save_reminders(&state.file_path, &reminders)?;

    Ok(())
}

// --------------------------------------------------
// Delete reminder
// --------------------------------------------------

#[tauri::command]
fn delete_reminder(
    id: u64,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut reminders = state
        .reminders
        .lock()
        .map_err(|_| "Failed to access reminders".to_string())?;

    reminders.retain(|reminder| reminder.id != id);

    save_reminders(&state.file_path, &reminders)?;

    Ok(())
}

// --------------------------------------------------
// Update reminder
// --------------------------------------------------

#[tauri::command]
fn update_reminder(
    id: u64,
    text: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut reminders = state
        .reminders
        .lock()
        .map_err(|_| "Failed to access reminders".to_string())?;

    let reminder = reminders
        .iter_mut()
        .find(|reminder| reminder.id == id);

    match reminder {
        Some(reminder) => {
            reminder.text = text;
        }

        None => {
            return Err("Reminder not found".to_string());
        }
    }

    save_reminders(&state.file_path, &reminders)?;

    Ok(())
}

// --------------------------------------------------
// Autostart status
// --------------------------------------------------

#[tauri::command]
fn get_autostart_status(
    app: tauri::AppHandle,
) -> Result<bool, String> {
    app.autolaunch()
        .is_enabled()
        .map_err(|error| error.to_string())
}

// --------------------------------------------------
// Enable autostart
// --------------------------------------------------

#[tauri::command]
fn enable_autostart(
    app: tauri::AppHandle,
) -> Result<(), String> {
    app.autolaunch()
        .enable()
        .map_err(|error| error.to_string())
}

// --------------------------------------------------
// Disable autostart
// --------------------------------------------------

#[tauri::command]
fn disable_autostart(
    app: tauri::AppHandle,
) -> Result<(), String> {
    app.autolaunch()
        .disable()
        .map_err(|error| error.to_string())?;

    Ok(())
}

// --------------------------------------------------
// Application startup
// --------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_data_dir = tauri::path::BaseDirectory::AppData;

    let state = tauri::Builder::default()

        // Register the autostart plugin
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .build()
        )

        // Application setup
        .setup(move |app| {
            // Find the application's data directory
            let app_data_path = app
                .path()
                .resolve("", app_data_dir)
                .map_err(|error| error.to_string())?;

            // Make sure the directory exists
            fs::create_dir_all(&app_data_path)
                .map_err(|error| error.to_string())?;

            // Location of our JSON file
            let file_path = app_data_path.join("reminders.json");

            // Load existing reminders
            let reminders = load_reminders(&file_path);

            // Give the reminders to the application state
            app.manage(AppState {
                reminders: Mutex::new(reminders),
                file_path,
            });

            Ok(())
        })

        // Register our commands
        .invoke_handler(tauri::generate_handler![
            get_reminders,
            add_reminder,
            delete_reminder,
            toggle_reminder,
            update_reminder,
            get_autostart_status,
            enable_autostart,
            disable_autostart
        ])

        // Start Tauri
        .run(tauri::generate_context!());

    if let Err(error) = state {
        eprintln!("Application error: {}", error);
    }
}
