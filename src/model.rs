use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime,NaiveDate, Utc};
use bigdecimal::BigDecimal;

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct UserModel {
    #[serde(rename = "User_ID")]
    pub user_id: i32,
    #[serde(rename = "Username")]
    pub username: String,
    #[serde(rename = "Password_Hash")]
    pub password_hash: String,
    #[serde(rename = "Email")]
    pub email: String,
    #[serde(rename = "Join_Date")]
    pub join_date: Option<DateTime<Utc>>,
    #[serde(rename = "Last_Online")]
    pub last_online: Option<DateTime<Utc>>,
    #[serde(rename = "Created_At")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "Updated_At")]
    pub updated_at: Option<DateTime<Utc>>,
}
#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct UpdatePassModel {
    #[serde(rename = "Password_Hash")]
    pub password_hash: String,
    #[serde(rename = "Updated_At")]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct GameModel {
    #[serde(rename = "Game_ID")]
    pub id: i32,
    #[serde(rename = "Title")]
    pub title: String,
    #[serde(rename = "Description")]
    pub description: Option<String>,
    #[serde(rename = "Genre")]
    pub genre: Option<String>,
    #[serde(rename = "Price_Paid")]
    pub price_paid: Option<BigDecimal>,
    #[serde(rename = "Barcode")]
    pub barcode: Option<String>,
    #[serde(rename = "Region")]
    pub region: Option<String>,
    #[serde(rename = "Images")]
    pub images: Option<String>,
    #[serde(rename = "Year_Released")]
    pub year_released: Option<NaiveDate>,
    #[serde(rename = "Date_Added")]
    pub date_added: Option<DateTime<Utc>>,
    #[serde(rename = "Last_Updated")]
    pub last_updated: Option<DateTime<Utc>>,
    #[serde(rename = "Developer")]
    pub developer: Option<String>,
    #[serde(rename = "Publisher")]
    pub publisher: Option<String>,
    #[serde(rename = "Rating")]
    pub rating: Option<BigDecimal>,
    #[serde(rename = "Status")]
    pub status: Option<String>,
    #[serde(rename = "Hours_Played")]
    pub hours_played: Option<i32>,
    #[serde(rename = "Platform_ID")]
    pub platform_id: i32,

}

#[derive(Debug, FromRow, Deserialize, Serialize)]
pub struct PlatformModel {
    #[serde(rename = "Platform_ID")]
    pub platform_id: i32,
    #[serde(rename = "Name")]
    pub name: Option<String>,
    #[serde(rename = "Short_Name")]
    pub short_name: Option<String>,
    #[serde(rename = "Manufacturer")]
    pub manufacturer: Option<String>,
}
