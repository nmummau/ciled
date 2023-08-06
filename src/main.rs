use warp::{Filter, Reply};
use serde::Deserialize;
use std::convert::Infallible;
use reqwest::Error;

#[derive(Debug, Deserialize)]
struct BuildStatus {
    build_id: String,
    status: String,
}

#[derive(Deserialize, Debug)]
struct WledResponse {
    state: WledState,
}

#[derive(Deserialize, Debug)]
struct WledState {
    on: bool,
}

async fn handle_webhook(build_status: BuildStatus) -> Result<impl Reply, Infallible> {
    println!("Received webhook: {:?}", build_status);

    println!("Build ID: {:?}", build_status.build_id);
    println!("Build Status: {:?}", build_status.status);

    match &build_status.status[..] {
        "Success" => set_led_color("green").await,
        "Failure" => set_led_color("red").await,
        _ => println!("Unknown status"),
    }

    Ok("Received")
}

async fn set_led_color(color: &str) {
    match change_led_color(color).await {
        Ok(response) => {
            if response.state.on {
                println!("LED color set to {} successfully", color);
            } else {
                println!("Failed to set LED color to {}", color);
            }
        }
        Err(e) => println!("Failed to call WLED API, error: {:?}", e),
    }
}

async fn change_led_color(color: &str) -> Result<WledResponse, Error> {
    let client = reqwest::Client::new();
    let color_rgb = match color {
        "red" => "255,0,0",
        "green" => "0,255,0",
        _ => "0,0,255",  // default to blue
    };

    let url = format!("http://192.168.10.50/json/state");
    let body = format!("{{\"on\": true, \"bri\": 255, \"seg\": {{\"col\": [[{}]]}}}}", color_rgb);

    println!("url: {:?}", url);
    println!("body: {:?}", body);

    let response = client.post(&url)
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await?
        .json::<WledResponse>()
        .await?;

    Ok(response)
}

#[tokio::main]
async fn main() {
    let webhook_route = warp::path!("webhook")
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::body::json())
        .and_then(handle_webhook);

    warp::serve(webhook_route).run(([127, 0, 0, 1], 3030)).await;
}
