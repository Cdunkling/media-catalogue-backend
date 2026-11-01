use serde::{Deserialize, Serialize};
use chrono::{DateTime, NaiveDate, Utc};
use bigdecimal::BigDecimal;

#[derive(Deserialize, Debug)]
pub struct FilterOptions {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, sqlx::Type, serde::Serialize, serde::Deserialize)]
#[sqlx(type_name = "ownership")]
pub enum Ownership {
    Owned,
    Wanted,
}

#[derive(Debug, Clone, Copy, PartialEq, sqlx::Type, serde::Serialize, serde::Deserialize)]
#[sqlx(type_name = "status")]
pub enum PlayStatus {
    Unplayed,
    Playing,
    Finished,
    Completed,
    #[sqlx(rename = "On Hold")]
    OnHold,
    Dropped,
}

#[derive(Debug, Clone, Copy, PartialEq, sqlx::Type, serde::Serialize, serde::Deserialize)]
#[sqlx(type_name = "contents")]
pub enum Contents {
    #[sqlx(rename = "CIB")]
    Cib,
    Loose,
    Sealed,

}

#[derive(Debug, Clone, Copy, PartialEq, sqlx::Type, serde::Serialize, serde::Deserialize)]
#[sqlx(type_name = "currency")]
pub enum Currency {
    #[sqlx(rename = "GBP")] Gbp,
    #[sqlx(rename = "JPY")] Jpy,
    #[sqlx(rename = "USD")] Usd,
    #[sqlx(rename = "EUR")] Eur,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameInput {
    #[serde(rename = "Title")]          pub title: String,
    #[serde(rename = "Description")]    pub description: Option<String>,
    #[serde(rename = "Genre")]          pub genre: Option<String>,
    #[serde(rename = "Price_Paid")]     pub price_paid: Option<BigDecimal>,
    #[serde(rename = "Ownership")]      pub ownership: Option<Ownership>,
    #[serde(rename = "Contents")]       pub contents: Option<Contents>,
    #[serde(rename = "Currency")]       pub currency: Option<Currency>,
    #[serde(rename = "Barcode")]        pub barcode: Option<String>,
    #[serde(rename = "Region")]         pub region: Option<String>,
    #[serde(rename = "Images")]         pub images: Option<String>,
    #[serde(rename = "Release_Date")]   pub release_date: Option<NaiveDate>,
    #[serde(rename = "Developer")]      pub developer: Option<String>,
    #[serde(rename = "Publisher")]      pub publisher: Option<String>,
    #[serde(rename = "Rating")]         pub rating: Option<BigDecimal>,
    #[serde(rename = "Status")]         pub status: Option<PlayStatus>,
    #[serde(rename = "Hours_Played")]   pub hours_played: Option<i32>,
    #[serde(rename = "Platform_ID")]    pub platform_id: i32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameUpdate {
    #[serde(rename = "Title")]         pub title: Option<String>,
    #[serde(rename = "Description")]   pub description: Option<String>,
    #[serde(rename = "Genre")]         pub genre: Option<String>,
    #[serde(rename = "Price_Paid")]    pub price_paid: Option<BigDecimal>,
    #[serde(rename = "Ownership")]     pub ownership: Option<Ownership>,
    #[serde(rename = "Contents")]      pub contents: Option<Contents>,
    #[serde(rename = "Currency")]      pub currency: Option<Currency>,
    #[serde(rename = "Barcode")]       pub barcode: Option<String>,
    #[serde(rename = "Region")]        pub region: Option<String>,
    #[serde(rename = "Images")]        pub images: Option<String>,
    #[serde(rename = "Release_Date")]  pub release_date: Option<NaiveDate>,
    #[serde(rename = "Developer")]     pub developer: Option<String>,
    #[serde(rename = "Publisher")]     pub publisher: Option<String>,
    #[serde(rename = "Rating")]        pub rating: Option<BigDecimal>,
    #[serde(rename = "Status")]        pub status: Option<PlayStatus>,
    #[serde(rename = "Hours_Played")]  pub hours_played: Option<i32>,
    #[serde(rename = "Platform_ID")]   pub platform_id: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct PlatformInput {
    #[serde(rename = "Name")]           pub name: String,
    #[serde(rename = "Short_Name")]     pub short_name: String,
    #[serde(rename = "Manufacturer")]   pub manufacturer: String,
}

#[derive(Debug, Deserialize)]
pub struct PlatformUpdate {
    #[serde(rename = "Name")]           pub name: String,
    #[serde(rename = "Short_Name")]     pub short_name: String,
    #[serde(rename = "Manufacturer")]   pub manufacturer: String,
}