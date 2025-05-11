use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GetAppList {
    #[serde(rename = "applist")]
    pub apps_list: AppsList,
}

#[derive(Debug, Deserialize)]
pub struct AppsList {
    pub apps: Vec<App>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "applist")]
pub struct App {
    #[serde(rename = "appid")]
    pub id: u32,
    pub name: String,
}
