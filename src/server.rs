use anyhow::Result;
use axum::{
    Json, Router,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
};
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
    data_next_reset: u64,
    node_location: String,
    os: String,
}

#[derive(Serialize, Clone)]
struct VpsInfo {
    veid: String,
    plan: String,
    ip_address: String,
    plan_monthly_data: u64,
    data_counter: u64,
    data_next_reset: u64,
    usage_percentage: f64,
    node_location: String,
    os: String,
}

impl VpsInfo {
    async fn get(veid: &str, api_key: &str) -> Result<Self> {
        let url = format!(
            "https://api.64clouds.com/v1/getServiceInfo?veid={}&api_key={}",
            veid, api_key
        );

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;

        let raw: BandwagonRawResponse = response.json().await?;

        let usage_percentage = if raw.plan_monthly_data > 0 {
            (raw.data_counter as f64 / raw.plan_monthly_data as f64) * 100.0
        } else {
            0.0
        };

        Ok(VpsInfo {
            veid: veid.to_string(),
            plan: raw.plan,
            ip_address: raw.ip_addresses.get(0).cloned().unwrap_or_default(),
            plan_monthly_data: raw.plan_monthly_data,
            data_counter: raw.data_counter,
            data_next_reset: raw.data_next_reset,
            usage_percentage,
            node_location: raw.node_location,
            os: raw.os,
        })
    }

    async fn all() -> Result<Vec<Self>> {
        let config = cfg()?;
        let futures = config
            .credentials
            .iter()
            .map(|(veid, key)| Self::get(veid, key));

        let mut results = Vec::new();
        for i in join_all(futures).await {
            results.push(i?);
        }
        Ok(results)
    }
}

fn r<T: IntoResponse>(v: Result<T>) -> impl IntoResponse {
    match v {
        Ok(v) => v.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", e)).into_response(),
    }
}

async fn get_vps_info() -> impl IntoResponse {
    r(VpsInfo::all().await.map(Json))
}

async fn get_vps_info_page() -> impl IntoResponse {
    async fn get() -> Result<String> {
        let infos = VpsInfo::all().await?;
        let config = cfg()?;
        let template = config.jinja.get_template("info-page")?;
        Ok(template.render(context! { vps_list => infos })?)
    }

    r(get().await.map(Html))
}

pub async fn run() -> Result<()> {
    let config = cfg()?;
    let app = Router::new()
        .route("/info", get(get_vps_info))
        .route("/", get(get_vps_info_page));

    let addr = &config.listen_addr;
    let listener = tokio::net::TcpListener::bind(addr).await?;

    println!("Server running on http://{}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
