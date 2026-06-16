// Templazy — Tauri v2 バックエンド
// データ（personal.json / tabs.json）を各OSユーザーのアプリデータ領域に保存する。
// HTTPサーバーは使わず、フロントから invoke('load'|'save', ...) で呼ばれる。

// リリース時に Windows でコンソール窓を出さない
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs;
use std::path::PathBuf;

use serde_json::Value;
use tauri::Manager;

// 初回起動時の初期テンプレート（社内向け内容に差し替え可能）。
// 既定は空配列。ビルド時にこのファイルへ seed を入れておくと初回に取り込まれる。
const SEED_TEMPLATES: &str = include_str!("../seed_templates.json");

/// 許可された固定ファイルのみを返す（パストラバーサル対策）。
fn resolve_path(app: &tauri::AppHandle, name: &str) -> Option<PathBuf> {
    let fname = match name {
        "personal" => "personal.json",
        "tabs" => "tabs.json",
        _ => return None,
    };
    let dir = app.path().app_data_dir().ok()?;
    Some(dir.join(fname))
}

/// データ読み込み。無ければ既定値（personal=[]、tabs={...}）を返す。
#[tauri::command]
fn load(app: tauri::AppHandle, name: String) -> Result<Value, String> {
    let path = resolve_path(&app, &name).ok_or("unknown resource")?;
    if !path.exists() {
        let def = if name == "tabs" {
            serde_json::json!({ "tabs": [], "activeTabId": null })
        } else {
            serde_json::json!([])
        };
        return Ok(def);
    }
    let txt = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&txt).map_err(|e| e.to_string())
}

/// データ保存。一時ファイルに書いてから rename（アトミック）。
#[tauri::command]
fn save(app: tauri::AppHandle, name: String, data: Value) -> Result<(), String> {
    let path = resolve_path(&app, &name).ok_or("unknown resource")?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let txt = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, txt).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())?;
    Ok(())
}

/// 初回起動時、personal.json が無ければ同梱 seed から作成する。
fn ensure_seed(app: &tauri::AppHandle) {
    let Some(path) = resolve_path(app, "personal") else { return };
    if path.exists() {
        return;
    }
    // seed が JSON 配列のときだけ取り込む
    if serde_json::from_str::<Value>(SEED_TEMPLATES)
        .map(|v| v.is_array())
        .unwrap_or(false)
    {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&path, SEED_TEMPLATES);
    }
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let handle = app.handle().clone();
            ensure_seed(&handle);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![load, save])
        .run(tauri::generate_context!())
        .expect("error while running Templazy");
}
