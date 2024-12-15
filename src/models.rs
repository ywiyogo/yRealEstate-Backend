use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ------------- User --------------------
#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Seller,
    Buyer,
    Owner,
    Tenant,
    Agent,
}

// Add this implementation
impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Convert enum variant to lowercase string
        match self {
            UserRole::Seller => write!(f, "seller"),
            UserRole::Buyer => write!(f, "buyer"),
            UserRole::Owner => write!(f, "owner"),
            UserRole::Tenant => write!(f, "tenant"),
            UserRole::Agent => write!(f, "agent"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewUser {
    pub email: String,
    pub password: String,
    pub full_name: String,
    pub phone: Option<String>,
    pub role: UserRole,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Option<i64>,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub full_name: String,
    pub phone: Option<String>,
    pub role: UserRole,
    pub verified: Option<bool>,
    pub profile_image_url: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub reset_token: Option<String>,
    pub reset_token_expires: Option<String>,
}

// ------------- Properties --------------------
#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum PropertyType {
    House,
    Apartment,
    Land,
    Commercial,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ListingType {
    Sale,
    Rent,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum PropertyStatus {
    Active,
    Pending,
    Sold,
    Rented,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ViewingStatus {
    Requested,
    Confirmed,
    Completed,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Property {
    pub id: Option<i64>,
    pub title: String,
    pub price: f64,
    pub description: Option<String>,
    pub location: String,
    pub bedrooms: Option<i64>,
    pub bathrooms: Option<i64>,
    pub square_feet: Option<f64>,
    pub property_type: PropertyType,
    pub listing_type: ListingType,
    pub status: PropertyStatus,
    pub owner_id: i64,
    pub agent_id: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PropertyImage {
    pub id: Option<i64>,
    pub property_id: i64,
    pub image_url: String,
    pub is_primary: Option<bool>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PropertyViewing {
    pub id: Option<i64>,
    pub property_id: i64,
    pub user_id: i64,
    pub viewing_date: String,
    pub status: ViewingStatus,
    pub notes: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Review {
    pub id: Option<i64>,
    pub property_id: i64,
    pub reviewer_id: i64,
    pub rating: i32,
    pub comment: Option<String>,
    pub created_at: Option<String>,
}
// ------------- Message --------------------
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Conversation {
    pub id: Option<i64>,
    pub property_id: i64,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: Option<i64>,
    pub conversation_id: i64,
    pub sender_id: i64,
    pub content: String,
    pub read: Option<bool>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewConversation {
    pub property_id: i64,
    pub participant_ids: Vec<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewMessage {
    pub sender_id: i64,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ConversationDetails {
    pub id: i64,
    pub property_id: i64,
    pub property_title: String,
    pub unread_count: i64,
    pub created_at: Option<String>,
}

// -------------- Authentication -----------

#[derive(Debug, Deserialize)]
pub struct LoginCredentials {
    pub email: String,
    pub password: String,
}
