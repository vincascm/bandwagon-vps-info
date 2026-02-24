use axum::{
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use anyhow::{Context, Result};
use futures::future::join_all;
use minijinja::context;
use serde::{Deserialize, Serialize};
use crate::statics::cfg;

#[derive(Deserialize)]
struct BandwagonRawResponse {
    plan: String,
    ip_addresses: Vec<String>,
    plan_monthly_data: u64,
    data_counter: u64,
}

#[derive(Serialize, Clone)]
struct VpsInfo {
    veid: String,
    plan: String,
    ip_address: String,
    plan_monthly_data: u64,
    data_counter: u64,
    usage_percentage: f64,
}

pub fn human_readable_size(size: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < units.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.2} {}", size, units[unit_idx])
}

async fn fetch_single_vps(veid: String, api_key: String) -> Result<VpsInfo> {
    let url = format!(
        "https://api.64clouds.com/v1/getServiceInfo?veid={}&api_key={}",
        veid, api_key
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .with_context(|| format!("Network error fetching VEID: {}", veid))?;

    let raw: BandwagonRawResponse = response
        .json()
        .await
        .with_context(|| format!("Parse error for VEID: {}", veid))?;

    let usage_percentage = if raw.plan_monthly_data > 0 {
        (raw.data_counter as f64 / raw.plan_monthly_data as f64) * 100.0
    } else {
        0.0
    };

    Ok(VpsInfo {
        veid,
        plan: raw.plan,
        ip_address: raw.ip_addresses.get(0).cloned().unwrap_or_default(),
        plan_monthly_data: raw.plan_monthly_data,
        data_counter: raw.data_counter,
        usage_percentage,
    })
}

async fn fetch_all_vps() -> Result<Vec<VpsInfo>> {
    let config = cfg()?;
    let futures = config
        .credentials
        .iter()
        .map(|(veid, key)| fetch_single_vps(veid.clone(), key.clone()));

    let results = join_all(futures).await;
    Ok(results.into_iter().filter_map(|r| r.ok()).collect())
}

async fn get_vps_info() -> impl IntoResponse {
    match fetch_all_vps().await {
        Ok(infos) => Json(infos).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response(),
    }
}

async fn get_vps_info_page() -> impl IntoResponse {
    let infos = match fetch_all_vps().await {
        Ok(infos) => infos,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response(),
    };

    let config = match cfg() {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Config error: {}", e)).into_response(),
    };

    let template = match config.jinja.get_template("info-page") {
        Ok(t) => t,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Template error: {}", e)).into_response(),
    };
    
    match template.render(context! { vps_list => infos }) {
        Ok(html) => Html(html).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Render error: {}", e)).into_response(),
    }
}

pub async fn run() -> Result<()> {
    let app = Router::new()
        .route("/info", get(get_vps_info))
        .route("/info-page", get(get_vps_info_page));

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await
        .with_context(|| format!("Failed to bind to {}", addr))?;
    
    println!("Server running on http://{}", addr);
    axum::serve(listener, app).await
        .context("Server error")?;
    
    Ok(())
}
