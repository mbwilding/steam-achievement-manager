use crate::contracts::{App, GetAppList};
use std::collections::HashMap;
use std::env;
use std::path::Path;
use tokio::fs;
use xmltree::Element;

#[cfg(windows)]
use winreg::enums::*;

#[cfg(windows)]
pub async fn get_app_list_library() -> Vec<App> {
    let steam_id3 = read_registry(r"Software\Valve\Steam\ActiveProcess", "ActiveUser");
    let profile_id = (1u64 << 56) | (1u64 << 52) | (1u64 << 32) | steam_id3 as u64;
    let url = format!(
        "https://steamcommunity.com/profiles/{}/games?xml=1",
        profile_id
    );
    fetch_and_parse(url).await
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub async fn get_app_list_library() -> Vec<App> {
    let home = if cfg!(target_os = "linux") {
        env::var("HOME").unwrap_or_else(|_| setting_failure())
    } else {
        dirs::home_dir()
            .and_then(|p| p.into_os_string().into_string().ok())
            .unwrap_or_else(|| setting_failure())
    };

    let file = if cfg!(target_os = "linux") {
        ".steam/steam/config/loginusers.vdf"
    } else {
        "Library/Application Support/Steam/config/loginusers.vdf"
    };
    let combined = Path::new(&home).join(file);

    let content = fs::read_to_string(&combined)
        .await
        .unwrap_or_else(|_| setting_failure());

    let steam_ids_line = content
        .lines()
        .find(|line| line.trim_start().starts_with("\"765"))
        .unwrap_or_else(|| setting_failure());

    let steam_ids = steam_ids_line.replace("\t", "").replace("\"", "");

    let profile_id: u64 = steam_ids
        .trim()
        .parse()
        .unwrap_or_else(|_| setting_failure());
    let url = format!(
        "https://steamcommunity.com/profiles/{}/games?xml=1",
        profile_id
    );

    fetch_and_parse(url).await
}

async fn fetch_and_parse(url: String) -> Vec<App> {
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .unwrap_or_else(|_| setting_failure());

    if !resp.status().is_success() {
        setting_failure();
    }
    let body = resp.text().await.unwrap_or_else(|_| setting_failure());
    let dict = parse_xml_to_dictionary(&body);
    if dict.is_empty() {
        setting_failure();
    }
    dict
}

fn setting_failure() -> ! {
    let error_msg = "Preparation required\n\n\
    Sign in to steam here: 'https://steamcommunity.com/my/edit/settings'\n\
    Set 'Game details' to 'Public'\n\n\
    Then re-run this program";
    eprintln!("Error: {}", error_msg);
    let mut _input = String::new();
    let _ = std::io::stdin().read_line(&mut _input);
    std::process::exit(1);
}

fn parse_xml_to_dictionary(xml: &str) -> Vec<App> {
    let root = Element::parse(xml.as_bytes()).unwrap_or_else(|err| {
        eprintln!("XML parsing error: {}", err);
        setting_failure();
    });

    let games_elem = root.get_child("games").unwrap_or_else(|| {
        eprintln!("Missing <games> element in XML");
        setting_failure();
    });

    let mut apps = vec![];
    for game in games_elem
        .children
        .iter()
        .filter_map(|child| child.as_element())
    {
        let name = game.get_child("name").and_then(|e| e.get_text());
        let app_id = game.get_child("appID").and_then(|e| e.get_text());
        if let (Some(name), Some(app_id)) = (name, app_id) {
            let id: u32 = app_id.parse().expect("app_id should be a valid u32");
            let name = name.to_string();
            apps.push(App { id, name });
        }
    }
    apps
}

#[cfg(windows)]
fn read_registry(base_path: &str, dword: &str) -> u32 {
    let hkcu = winreg::RegKey::predef(HKEY_CURRENT_USER);
    let subkey = hkcu
        .open_subkey_with_flags(base_path, KEY_READ)
        .unwrap_or_else(|_| {
            eprintln!("Failed to open registry key: {}", base_path);
            setting_failure();
        });
    let value: u32 = subkey.get_value(dword).unwrap_or_else(|_| {
        eprintln!("Failed to read registry value: {}", dword);
        setting_failure();
    });
    value
}

pub async fn get_app_list_all_vec() -> Vec<App> {
    let http = reqwest::Client::new();

    let response = http
        .get("https://api.steampowered.com/ISteamApps/GetAppList/v2")
        .send()
        .await
        .expect("Steam GetAppList API unavailable");

    response
        .json::<GetAppList>()
        .await
        .expect("Unable to deserialise response from Steam GetAppList API")
        .apps_list
        .apps
}

pub async fn get_app_list_all_hash() -> HashMap<u32, String> {
    get_app_list_all_vec()
        .await
        .into_iter()
        .map(|app| (app.id, app.name))
        .collect()
}
