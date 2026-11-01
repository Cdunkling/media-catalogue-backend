use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime,NaiveDate, Utc};
use bigdecimal::BigDecimal;
use crate::schema::{Currency,Ownership,Contents,PlayStatus};

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct GameModel {
    #[serde(rename = "Game_ID")]       pub id: i32,
    #[serde(rename = "Title")]         pub title: String,
    #[serde(rename = "Description")]   pub description: Option<String>,
    #[serde(rename = "Genre")]         pub genre: Option<String>,
    #[serde(rename = "Price_Paid")]    pub price_paid: Option<BigDecimal>,
    #[serde(rename = "Currency")]      pub currency: Currency,
    #[serde(rename = "Ownership")]     pub ownership: Ownership,
    #[serde(rename = "Contents")]      pub contents: Contents,
    #[serde(rename = "Barcode")]       pub barcode: Option<String>,
    #[serde(rename = "Region")]        pub region: Option<String>,
    #[serde(rename = "Images")]        pub images: Option<String>,
    #[serde(rename = "Release_Date")] pub release_date: Option<NaiveDate>,
    #[serde(rename = "Developer")]     pub developer: Option<String>,
    #[serde(rename = "Publisher")]     pub publisher: Option<String>,
    #[serde(rename = "Rating")]        pub rating: Option<BigDecimal>,
    #[serde(rename = "Status")]        pub status: PlayStatus,
    #[serde(rename = "Hours_Played")]  pub hours_played: Option<i32>,
    #[serde(rename = "Date_Added")]    pub date_added: DateTime<Utc>,
    #[serde(rename = "Last_Updated")]  pub last_updated: DateTime<Utc>,
    #[serde(rename = "Platform_ID")]   pub platform_id: i32,
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct PlatformModel {
    #[serde(rename = "Platform_ID")]    pub id: i32,
    #[serde(rename = "Name")]           pub name: Option<String>,
    #[serde(rename = "Short_Name")]     pub short_name: Option<String>,
    #[serde(rename = "Manufacturer")]   pub manufacturer: Option<String>,
}
