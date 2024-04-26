use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StripePaymentIntentResponse {
  pub amount_received: usize,
  pub client_secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentIntentResponse {
  pub client_secret: String,
}

pub struct Stripe {
  client: reqwest::Client,
}

impl Stripe {
  pub fn new(secret_key: &str) -> Self {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
      "Authorization",
      format!("Bearer {}", secret_key)
        .parse()
        .expect("failed to parse header"),
    );

    Self {
      client: reqwest::Client::builder()
        .default_headers(headers)
        .build()
        .expect("failed to create stripe client"),
    }
  }

  pub async fn create_payment_intent(
    &self,
    amount: usize,
    mode: String,
  ) -> Result<StripePaymentIntentResponse, crate::http::PayError> {
    let mut amount = amount;

    if mode == "card" {
      amount += get_fee(amount);
    };

    let payment_method = match mode.as_str() {
      "card" => "card".to_string(),
      "ach" => "us_bank_account".to_string(),
      _ => return Err(crate::http::PayError::BadRequest("invalid payment method".to_string())),
    };

    tracing::info!("creating payment intent for amount {} with mode {}", amount / 100, mode);

    let create_intent = [
      ("amount", amount.to_string()),
      ("currency", "usd".into()),
      ("payment_method_types[0]", payment_method),
    ];

    tracing::trace!("request: {:?}", create_intent);

    let res = self
      .client
      .post("https://api.stripe.com/v1/payment_intents")
      .form(&create_intent)
      .send()
      .await?;

    if res.status().is_success() {
      tracing::info!("payment intent created");
    } else {
      tracing::error!("failed to create payment intent: {:?}", res.json().await?);
      return Err(crate::http::PayError::InternalError(
        "failed to create payment intent".to_string(),
      ));
    }

    let intent: StripePaymentIntentResponse = res.json().await?;

    Ok(intent)
  }
}

fn get_fee(amount: usize) -> usize {
  ((amount as f32 + 30.0) / (1.0 - 0.029)).round() as usize - amount
}
