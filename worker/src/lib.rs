#![feature(let_chains)]

use std::{collections::HashMap, sync::Arc};

use axum::{extract, http::HeaderValue, response, routing, Router};
use stripe::PaymentIntentResponse;
use tower_http::cors::CorsLayer;
use tower_service::Service;

mod http;
mod monitoring;
mod stripe;
mod wave;

#[derive(Debug)]
struct AppState {
  waveapps_business_id: String,
  waveapps_access_token: String,
  stripe_secret_key: String,
  redirect_uri: String,
  cors_origin: String,
}

#[worker::event(fetch, respond_with_errors)]
async fn fetch(
  req: worker::HttpRequest,
  env: worker::Env,
  _ctx: worker::Context,
) -> worker::Result<axum::http::Response<axum::body::Body>> {
  tracing::info!("request received, starting up");
  tracing::trace!(request=?req, "handling request");

  let state = AppState {
    waveapps_business_id: env
      .secret("WAVEAPPS_BUSINESS_ID")
      .expect("WAVEAPPS_BUSINESS_ID must be set")
      .to_string(),
    waveapps_access_token: env
      .secret("WAVEAPPS_ACCESS_TOKEN")
      .expect("WAVEAPPS_ACCESS_TOKEN must be set")
      .to_string(),
    stripe_secret_key: env
      .secret("STRIPE_SECRET_KEY")
      .expect("STRIPE_SECRET_KEY must be set")
      .to_string(),
    redirect_uri: env
      .secret("REDIRECT_URI")
      .expect("REDIRECT_URI must be set")
      .to_string(),
    cors_origin: env.secret("CORS_ORIGIN").expect("CORS_ORIGIN must be set").to_string(),
  };

  // #[cfg(debug_assertions)]
  // tracing::trace!(state = ?state, "state loaded");

  Ok(router(state).call(req).await?)
}

fn router(state: AppState) -> Router {
  let state = Arc::new(state);

  Router::new()
    .route("/invoice/:number", routing::get(get_invoice))
    .route("/invoice/:id/pay/:mode", routing::get(create_payment_intent))
    .route("/stripe/callback", routing::get(stripe_callback))
    .layer(CorsLayer::new().allow_origin(state.cors_origin.parse::<HeaderValue>().expect("invalid cors origin")))
    .with_state(state)
}

#[axum_wasm_macros::wasm_compat]
async fn get_invoice(
  extract::State(state): extract::State<Arc<AppState>>,
  extract::Path(number): extract::Path<String>,
) -> Result<http::PayJson<wave::Invoice>, http::PayError> {
  tracing::info!("handling get_invoice request for invoice number {number}");
  let invoice = wave::Invoice::find(number, &state).await?;

  Ok(http::PayJson(invoice))
}

#[axum_wasm_macros::wasm_compat]
async fn create_payment_intent(
  extract::State(state): extract::State<Arc<AppState>>,
  extract::Path((invoice_id, mode)): extract::Path<(String, String)>,
) -> Result<http::PayJson<stripe::PaymentIntentResponse>, http::PayError> {
  tracing::info!("handling create_payment_intent request for invoice {invoice_id} with method {mode}");
  let amount_due = wave::AmountDue::get(invoice_id, &state).await?;
  tracing::trace!(amount_due = ?amount_due, "amount due");

  let stripe = stripe::Stripe::new(&state.stripe_secret_key);
  let payment_intent = stripe.create_payment_intent(amount_due.value, mode).await?;
  tracing::trace!(payment_intent = ?payment_intent, "payment intent");

  tracing::info!("payment intent created");

  Ok(http::PayJson(PaymentIntentResponse {
    client_secret: payment_intent.client_secret,
  }))
}

#[axum_wasm_macros::wasm_compat]
async fn stripe_callback(
  extract::State(state): extract::State<Arc<AppState>>,
  extract::Query(query): extract::Query<HashMap<String, String>>,
) -> Result<response::Redirect, http::PayError> {
  tracing::info!("handling stripe_callback request");
  tracing::trace!(query = ?query, "query params");

  Ok(response::Redirect::temporary(&state.redirect_uri))
}
