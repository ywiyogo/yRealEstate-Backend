use crate::models::{ListingType, Property, PropertyStatus, PropertyType};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
};
use sqlx::SqlitePool;

pub async fn list_properties(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Property>>, StatusCode> {
    let properties = sqlx::query_as!(
        Property,
        r#"
        SELECT 
            id, title, price, description, location, 
            bedrooms, bathrooms, square_feet,
            property_type as "property_type: PropertyType",
            listing_type as "listing_type: ListingType",
            status as "status: PropertyStatus",
            owner_id, agent_id, created_at, updated_at
        FROM properties
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(properties))
}

pub async fn create_property(
    State(pool): State<SqlitePool>,
    Json(property): Json<Property>,
) -> Result<Json<Property>, StatusCode> {
    let created_property = sqlx::query_as!(
        Property,
        r#"
        INSERT INTO properties (
            title, price, description, location,
            bedrooms, bathrooms, square_feet,
            property_type, listing_type, status
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING id, title, price, description, location,
            bedrooms, bathrooms, square_feet,
            property_type as "property_type: PropertyType",
            listing_type as "listing_type: ListingType",
            status as "status: PropertyStatus",
            owner_id, agent_id, created_at, updated_at
        "#,
        property.title,
        property.price,
        property.description,
        property.location,
        property.bedrooms,
        property.bathrooms,
        property.square_feet,
        property.property_type,
        property.listing_type,
        property.status
    )
    .fetch_one(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(created_property))
}

pub async fn get_property(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<Property>, StatusCode> {
    let property = sqlx::query_as!(
        Property,
        r#"
        SELECT id, title, price, description, location,
               bedrooms, bathrooms, square_feet,
               property_type as "property_type: PropertyType",
               listing_type as "listing_type: ListingType",
               status as "status: PropertyStatus",
               owner_id, agent_id, created_at, updated_at
        FROM properties
        WHERE id = ?
        "#,
        id
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(property))
}
