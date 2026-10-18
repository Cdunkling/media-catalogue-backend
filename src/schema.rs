use serde::{Deserialize, Serialize};
use chrono::{DateTime, NaiveDate, Utc};
use bigdecimal::BigDecimal;

#[derive(Deserialize, Debug)]
pub struct FilterOptions {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserInput {
    #[serde(rename = "Username")]   pub username: String,
    #[serde(rename = "Password")]   pub password: String,
    #[serde(rename = "Email")]      pub email: String,
}

#[derive(Deserialize)]
pub struct PasswordChange {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserPatch {
    pub username: Option<String>,
    pub email: Option<String>,
}

#[derive(Serialize)]
pub struct UserOut {
    pub user_id: i32,
    pub username: String,
    pub email: String,
    pub join_date: DateTime<Utc>,
    pub last_online: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GameInput {
    #[serde(rename = "Title")]          pub title: String,
    #[serde(rename = "Description")]    pub description: Option<String>,
    #[serde(rename = "Genre")]          pub genre: Option<String>,
    #[serde(rename = "Price_Paid")]     pub price_paid: Option<BigDecimal>,
    #[serde(rename = "Barcode")]        pub barcode: Option<String>,
    #[serde(rename = "Region")]         pub region: Option<String>,
    #[serde(rename = "Images")]         pub images: Option<String>,
    #[serde(rename = "Year_Released")]  pub year_released: Option<NaiveDate>,
    #[serde(rename = "Developer")]      pub developer: Option<String>,
    #[serde(rename = "Publisher")]      pub publisher: Option<String>,
    #[serde(rename = "Rating")]         pub rating: Option<BigDecimal>,
    #[serde(rename = "Status")]         pub status: Option<String>,
    #[serde(rename = "Hours_Played")]   pub hours_played: Option<i32>,
    #[serde(rename = "Platform_ID")]    pub platform_id: i32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GamePatch {
    #[serde(rename = "Title")]         pub title: Option<String>,
    #[serde(rename = "Platform_ID")]   pub platform_id: Option<i32>,
    #[serde(rename = "Description")]   pub description: Option<String>,
    #[serde(rename = "Genre")]         pub genre: Option<String>,
    #[serde(rename = "Price_Paid")]    pub price_paid: Option<BigDecimal>,
    #[serde(rename = "Barcode")]       pub barcode: Option<String>,
    #[serde(rename = "Region")]        pub region: Option<String>,
    #[serde(rename = "Images")]        pub images: Option<String>,
    #[serde(rename = "Year_Released")] pub year_released: Option<NaiveDate>,
    #[serde(rename = "Developer")]     pub developer: Option<String>,
    #[serde(rename = "Publisher")]     pub publisher: Option<String>,
    #[serde(rename = "Rating")]        pub rating: Option<BigDecimal>,
    #[serde(rename = "Status")]        pub status: Option<String>,
    #[serde(rename = "Hours_Played")]  pub hours_played: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct PlatformInput {
    #[serde(rename = "Name")]           pub name: String,
    #[serde(rename = "Short_Name")]     pub short_name: String,
    #[serde(rename = "Manufacturer")]   pub manufacturer: String,
}