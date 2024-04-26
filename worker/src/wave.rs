use graphql_client::{GraphQLQuery, Response as GraphQLResponse};
use serde::{Deserialize, Serialize};

use crate::{http::PayError, AppState};

const WAVEAPPS_URL: &str = "https://gql.waveapps.com/graphql/public";

#[derive(GraphQLQuery)]
#[graphql(
  schema_path = "graphql/schema.graphql.json",
  query_path = "graphql/find-invoice.graphql",
  response_derives = "Debug"
)]
pub struct FindInvoice;

#[derive(GraphQLQuery)]
#[graphql(
  schema_path = "graphql/schema.graphql.json",
  query_path = "graphql/get-invoice.graphql",
  response_derives = "Debug"
)]
pub struct GetInvoice;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Customer {
  pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmountDue {
  pub value: usize,
}

impl TryFrom<String> for AmountDue {
  type Error = crate::http::PayError;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    tracing::trace!("trying to convert amount due from string");

    Ok(Self {
      value: value
        .replace(&['.', ','][..], "")
        .parse::<usize>()
        .map_err(|_| PayError::DeserializeError("failed to deserialize invoice amount".into()))?,
    })
  }
}

impl AmountDue {
  pub async fn get(invoice_id: String, state: &AppState) -> Result<Self, crate::http::PayError> {
    tracing::info!("getting amount due for invoice {}", &invoice_id);

    let query = GetInvoice::build_query(get_invoice::Variables {
      business_id: state.waveapps_business_id.clone(),
      invoice_id,
    });

    let res = reqwest::Client::new()
      .post(WAVEAPPS_URL)
      .header("Authorization", format!("Bearer {}", state.waveapps_access_token))
      .json(&query)
      .send()
      .await?;
    tracing::trace!("get invoice response: {:?}", res);

    let body: GraphQLResponse<<GetInvoice as graphql_client::GraphQLQuery>::ResponseData> = res.json().await?;

    let amount_due = body
      .data
      .ok_or(PayError::DeserializeError(
        "failed to deserialize amount_due request data".into(),
      ))?
      .business
      .ok_or(PayError::DeserializeError("failed to deserialize business data".into()))?
      .invoice
      .ok_or(PayError::DeserializeError("failed to deserialize invoice data".into()))?
      .amount_due
      .value;

    AmountDue::try_from(amount_due)
  }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Invoice {
  pub id: String,
  pub pdf_url: String,
  pub invoice_number: String,
  pub customer: Customer,
  pub amount_due: AmountDue,
}

impl Invoice {
  pub async fn find(number: String, state: &AppState) -> Result<Invoice, crate::http::PayError> {
    tracing::info!("finding invoice {}", number);

    let query = FindInvoice::build_query(find_invoice::Variables {
      business_id: state.waveapps_business_id.clone(),
      invoice_number: number.clone(),
    });

    let res = reqwest::Client::new()
      .post(WAVEAPPS_URL)
      .header("Authorization", format!("Bearer {}", state.waveapps_access_token))
      .json(&query)
      .send()
      .await?;
    tracing::trace!("find invoice response: {:?}", res);

    let body: GraphQLResponse<<FindInvoice as graphql_client::GraphQLQuery>::ResponseData> = res.json().await?;
    tracing::trace!("body acquired: {:?}", body);

    let invoice = Self::try_from(body)?;
    tracing::trace!("invoice: {:?}", invoice);

    if invoice.invoice_number != number {
      tracing::info!("invoice number does not match - mapping to not found");
      return Err(PayError::NotFound(format!("invoice {} not found", number)));
    }

    tracing::info!("found invoice {}", number);

    Ok(invoice)
  }
}

impl TryFrom<GraphQLResponse<<FindInvoice as graphql_client::GraphQLQuery>::ResponseData>> for Invoice {
  type Error = crate::http::PayError;

  fn try_from(
    value: GraphQLResponse<<FindInvoice as graphql_client::GraphQLQuery>::ResponseData>,
  ) -> Result<Self, crate::http::PayError> {
    tracing::trace!("trying to convert invoice from graphql response");
    let invoice = value
      .data
      .ok_or(PayError::DeserializeError(
        "failed to deserialize invoice response".into(),
      ))?
      .business
      .ok_or(PayError::DeserializeError(
        "failed to deserialize invoice business".into(),
      ))?
      .invoices
      .ok_or(PayError::DeserializeError("failed to deserialize invoice array".into()))?
      .edges
      .into_iter()
      .nth(0)
      .ok_or(PayError::DeserializeError(
        "failed to deserialize invoice itself".into(),
      ))?
      .node;

    Ok(Self {
      id: invoice.id,
      pdf_url: invoice.pdf_url,
      invoice_number: invoice.invoice_number,
      customer: Customer {
        name: invoice.customer.name,
      },
      amount_due: AmountDue::try_from(invoice.amount_due.value)?,
    })
  }
}
