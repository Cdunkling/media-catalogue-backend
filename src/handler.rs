use crate::{
    AppState,
    schema::FilterOptions,
    model::GameModel,
    model::PlatformModel,
    schema::{GameInput, GameUpdate, PlatformInput,Currency,Ownership,Contents,PlayStatus}
};
use actix_web::{delete, get, patch, post, web, HttpResponse, Responder};
use serde_json::json;
use crate::schema::PlatformUpdate;

#[get("/healthchecker")]
async fn health_checker_handler() -> impl Responder {
    const MESSAGE: &str = "Build API with Rust, SQLX, Postgres,and Actix Web";

    HttpResponse::Ok().json(json!({"status": "success","message": MESSAGE}))
}

// game Handlers


#[get("/games")]
pub async fn game_list_handler(
    opts: web::Query<FilterOptions>,
    data: web::Data<AppState>,
) -> impl Responder {
    let limit = opts.limit.unwrap_or(10).clamp(1, 100);
    let page = opts.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;

    let result = sqlx::query_as!(
        GameModel,
        r#"SELECT
            id, title, description, genre, price_paid,
            ownership as "ownership!: Ownership",
            contents as "contents!: Contents",
            currency as "currency!: Currency",
            barcode, region, images, release_date, developer, publisher, rating,
            current_status as "status!: PlayStatus",
            hours_played, date_added, last_updated, platform_id
        FROM games ORDER BY id LIMIT $1 OFFSET $2"#,
        limit as i32,
        offset as i64
    )
        .fetch_all(&data.db)
        .await;

    match result {
        Ok(games) => HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "results": games.len(),
            "page": page,
            "limit": limit,
            "data": { "games": games }
        })),
        Err(e) => {
            eprintln!("failed to list games: {e}"); // or tracing::error!
            HttpResponse::InternalServerError().json(serde_json::json!({
                "status": "error",
                "message": "failed to fetch games"
            }))
        }
    }
}

#[post("/games")]
async fn create_game_handler(
    body: web::Json<GameInput>,
    data: web::Data<AppState>,
) -> impl Responder {
    if body.title.trim().is_empty() {
        return HttpResponse::BadRequest()
            .json(json!({"status": "fail", "message": "Title must not be blank"}));
    }
    let query_result = sqlx::query_as!(
        GameModel,
        r#"INSERT INTO games (
            title, description, genre, price_paid, ownership, contents, currency,
            barcode, region, images, release_date, developer, publisher, rating,
            current_status, hours_played, date_added, last_updated, platform_id
        )
        VALUES (
            $1, $2, $3, $4,
            COALESCE($5::ownership, 'Owned'),
            COALESCE($6::contents, 'CIB'),
            COALESCE($7::currency, 'GBP'),
            $8, $9, $10, $11, $12, $13, $14,
            COALESCE($15::status, 'Unplayed'),
            COALESCE($16, 0),
            CURRENT_TIMESTAMP, CURRENT_TIMESTAMP,
            $17
        )
        RETURNING
            id, title, description, genre, price_paid,
            ownership as "ownership!: Ownership",
            contents as "contents!: Contents",
            currency as "currency!: Currency",
            barcode, region, images, release_date, developer, publisher, rating,
            current_status as "status!: PlayStatus",
            hours_played, date_added, last_updated, platform_id"#,
        body.title,
        body.description,
        body.genre,
        body.price_paid,
        body.ownership as Option<Ownership>,
        body.contents as Option<Contents>,
        body.currency as Option<Currency>,
        body.barcode,
        body.region,
        body.images,
        body.release_date,
        body.developer,
        body.publisher,
        body.rating,
        body.status as Option<PlayStatus>,
        body.hours_played,
        body.platform_id,
    )
        .fetch_one(&data.db)
        .await;

    match query_result {
        Ok(game) => HttpResponse::Created()
            .json(json!({"status": "success", "data": {"game": game}})),

        Err(sqlx::Error::Database(e)) if e.code().as_deref() == Some("23505") => {
            HttpResponse::Conflict()
                .json(json!({"status": "fail", "message": "Game already exists"}))
        }

        Err(sqlx::Error::Database(e)) if e.code().as_deref() == Some("23503") => {
            HttpResponse::UnprocessableEntity()
                .json(json!({"status": "fail", "message": "Platform does not exist"}))
        }

        Err(e) => {
            eprintln!("failed to create game: {e}");   // full detail stays in your server log
            HttpResponse::InternalServerError()
                .json(json!({"status": "error", "message": "failed to create game"}))
        }
    }
}

/// Retrieves a single game by their ID.
///
/// This handler fetches a game from the database using the provided game ID from the URL path.
/// If the game is found, it returns a JSON response with the game data and a success status.
/// If the game is not found, it returns a 404 Not Found response with an error message.
///
/// # Parameters
///
/// * `path` - A `web::Path<i32>` containing the game ID extracted from the URL path parameter.
/// * `data` - A `web::Data<AppState>` containing the shared application state, including the database connection pool.
///
/// # Returns
///
/// Returns an `impl Responder` which will be one of:
/// * `HttpResponse::Ok` with JSON containing the game data if the game is found.
/// * `HttpResponse::NotFound` with JSON containing an error message if the game does not exist.
#[get("/games/{id}")]
async fn get_game_handler(path: web::Path<i32>, data: web::Data<AppState>) -> impl Responder {
    let id = path.into_inner();
    let result = sqlx::query_as!(
        GameModel,
        r#"SELECT
        id, title, description, genre, price_paid,
        ownership as "ownership!: Ownership",
        contents as "contents!: Contents",
        currency as "currency!: Currency",
        barcode, region, images, release_date, developer, publisher, rating,
        current_status as "status!: PlayStatus",
        hours_played, date_added, last_updated, platform_id
        FROM games WHERE ID = $1"#,
        id
    )
    .fetch_optional(&data.db)
    .await;

    match result {
        Ok(Some(game)) => HttpResponse::Ok()
            .json(json!({"status": "success", "data": {"game": game}})),
        Ok(None) => HttpResponse::NotFound()
            .json(json!({"status": "fail", "message": format!("game with ID: {id} not found")})),
        Err(e) => {
            eprintln!("failed to get game {id}: {e}");
            HttpResponse::InternalServerError()
                .json(json!({"status": "error", "message": "failed to fetch game"}))
        }
    }
}

#[patch("/games/{id}")]
async fn edit_game_handler(
    path: web::Path<i32>,
    body: web::Json<GameUpdate>,
    data: web::Data<AppState>,
) -> impl Responder {
    let id = path.into_inner();
    let result = sqlx::query_as!(
        GameModel,
        r#"UPDATE games SET
            title          = COALESCE($1, title),
            description    = COALESCE($2, description),
            genre          = COALESCE($3, genre),
            price_paid     = COALESCE($4, price_paid),
            ownership      = COALESCE($5::ownership, ownership),
            contents       = COALESCE($6::contents, contents),
            currency       = COALESCE($7::currency, currency),
            barcode        = COALESCE($8, barcode),
            region         = COALESCE($9, region),
            images         = COALESCE($10, images),
            release_date  = COALESCE($11, release_date),
            developer      = COALESCE($12, developer),
            publisher      = COALESCE($13, publisher),
            rating         = COALESCE($14, rating),
            current_status = COALESCE($15::status, current_status),
            hours_played   = COALESCE($16, hours_played),
            platform_id    = COALESCE($17, platform_id),
            last_updated   = CURRENT_TIMESTAMP
        WHERE id = $18
        RETURNING
            id, title, description, genre, price_paid,
            ownership as "ownership!: Ownership",
            contents as "contents!: Contents",
            currency as "currency!: Currency",
            barcode, region, images, release_date, developer, publisher, rating,
            current_status as "status!: PlayStatus",
            hours_played, date_added, last_updated, platform_id"#,
        body.title, body.description, body.genre, body.price_paid,
        body.ownership as Option<Ownership>,
        body.contents as Option<Contents>,
        body.currency as Option<Currency>,
        body.barcode, body.region, body.images, body.release_date,
        body.developer, body.publisher, body.rating,
        body.status as Option<PlayStatus>,
        body.hours_played, body.platform_id,
        id,
    )
    .fetch_optional(&data.db)
    .await;

    match result {
        Ok(Some(game)) => HttpResponse::Ok()
            .json(serde_json::json!({"status": "success","data": {"game":game}})),
        Ok(None) => HttpResponse::NotFound()
            .json(serde_json::json!({"status": "fail","message": "Game not found"})),
        Err(sqlx::Error::Database(e)) if e.code().as_deref() == Some("23505") => {
            HttpResponse::Conflict()
                .json(serde_json::json!({"status": "fail","message": "Game already exists"}))
        }

        Err(sqlx::Error::Database(e)) if e.code().as_deref() == Some("23503") => {
            HttpResponse::UnprocessableEntity()
                .json(json!({"status": "fail", "message": "Platform does not exist"}))
        }
        Err(e) => {
            eprintln!("failed to update ...: {e}");
            HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "error", "message": "Internal error"}))

        }
    }
}

#[delete("/games/{id}")]
async fn delete_game_handler(
    path: web::Path<i32>,
    data: web::Data<AppState>,
) -> impl Responder {
    let game_id = path.into_inner();

    match sqlx::query!("DELETE FROM games WHERE id = $1", game_id)
        .execute(&data.db)
        .await
    {
        Ok(res) if res.rows_affected() == 0 => HttpResponse::NotFound()
            .json(serde_json::json!({"status": "fail", "message": "Game not found"})),
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(sqlx::Error::Database(e)) if e.code().as_deref() == Some("23503") => {
            HttpResponse::Conflict().json(serde_json::json!({"status": "fail","message": "Game still has data that references it"}))
        }
        Err(e) => {
            eprintln!("error deleting game: {e:?}");
            HttpResponse::InternalServerError()
                .json(json!({"status": "error", "message": "Internal error"}))
        }
    }
}


#[get("/platforms")]
pub async fn platform_list_handler(
    opts: web::Query<FilterOptions>,
    data: web::Data<AppState>,
) -> impl Responder {
    let limit = opts.limit.unwrap_or(100).clamp(1, 100);
    let page = opts.page.unwrap_or(1).max(1);
    let offset = (page - 1) * limit;

        let query_result = sqlx::query_as!(
            PlatformModel,
            "SELECT * FROM platforms ORDER by id LIMIT $1 OFFSET $2",
            limit as i32,
            offset as i32
        )
        .fetch_all(&data.db)
        .await;

    match query_result {
        Ok(platform) => HttpResponse::Created()
            .json(json!({"status": "success", "data": {"platform": platform}})),

        Err(e) => {
            eprintln!("failed to list platforms: {e}");
            HttpResponse::InternalServerError()
                .json(json!({"status": "error", "message": "failed to list platforms"}))
        }
    }
}

#[get("/platforms/{id}")]
async fn get_platform_handler(path: web::Path<i32>, data: web::Data<AppState>) -> impl Responder {
    let id = path.into_inner();
    let result = sqlx::query_as!(
        PlatformModel,
        r#"SELECT
        id, name, short_name, manufacturer
        FROM platforms WHERE ID = $1"#,
        id
    )
        .fetch_optional(&data.db)
        .await;

    match result {
        Ok(Some(platform)) => HttpResponse::Ok()
            .json(json!({"status": "success", "data": {"platform": platform}})),
        Ok(None) => HttpResponse::NotFound()
            .json(json!({"status": "fail", "message": format!("platform with ID: {id} not found")})),
        Err(e) => {
            eprintln!("failed to get platform {id}: {e}");
            HttpResponse::InternalServerError()
                .json(json!({"status": "error", "message": "failed to fetch platform"}))
        }
    }
}

#[post("/platforms")]
async fn create_platform_handler(
    body: web::Json<PlatformInput>,
    data: web::Data<AppState>,
) -> impl Responder {
    if body.name.trim().is_empty() || body.short_name.trim().is_empty() {
        return HttpResponse::BadRequest()
            .json(json!({"status": "fail", "message": "Name and short name must not be blank"}));
    }
    let query_result = sqlx::query_as!(
    PlatformModel,
    r#"INSERT INTO platforms (
        name,short_name,manufacturer
    )
    VALUES ($1, $2, $3)
    RETURNING *"#,
    body.name,
    body.short_name,
    body.manufacturer,
)
        .fetch_one(&data.db)
        .await;

    match query_result {
        Ok(platform) => HttpResponse::Created()
            .json(json!({"status": "success", "data": {"platform": platform}})),

        Err(sqlx::Error::Database(e)) if e.code().as_deref() == Some("23505") => {
            HttpResponse::Conflict()
                .json(json!({"status": "fail", "message": "Platform already exists"}))
        }

        Err(e) => {
            eprintln!("failed to create platform: {e}");
            HttpResponse::InternalServerError()
                .json(json!({"status": "error", "message": "failed to create platform"}))
        }
    }
}

#[patch("/platforms/{id}")]
async fn edit_platform_handler(
    path: web::Path<i32>,
    body: web::Json<PlatformUpdate>,
    data: web::Data<AppState>,
) -> impl Responder {
    let id = path.into_inner();
    let result = sqlx::query_as!(
        PlatformModel,
        r#"UPDATE platforms SET
            name          = COALESCE($1, name),
            short_name   = COALESCE($2, short_name),
            manufacturer    = COALESCE($3, manufacturer)
        WHERE id = $4
        RETURNING
            id, name, short_name, manufacturer"#,
        body.name, body.short_name, body.manufacturer,
        id,
    )
        .fetch_optional(&data.db)
        .await;

    match result {
        Ok(Some(platform)) => HttpResponse::Ok()
            .json(serde_json::json!({"status": "success","data": {"platform":platform}})),
        Ok(None) => HttpResponse::NotFound()
            .json(serde_json::json!({"status": "fail","message": "Platform not found"})),
        Err(sqlx::Error::Database(e)) if e.code().as_deref() == Some("23505") => {
            HttpResponse::Conflict()
                .json(serde_json::json!({"status": "fail","message": "Platform already exists"}))
        }
        Err(e) => {
            eprintln!("failed to update ...: {e}");
            HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "error", "message": "Internal error"}))
        }
    }
}

#[delete("/platforms/{id}")]
async fn delete_platform_handler(
    path: web::Path<i32>,
    data: web::Data<AppState>,
) -> impl Responder {
    let id = path.into_inner();

    match sqlx::query!("DELETE FROM platforms WHERE id = $1", id)
        .execute(&data.db)
        .await
    {
        Ok(res) if res.rows_affected() == 0 => HttpResponse::NotFound()
            .json(serde_json::json!({"status": "fail", "message": "Platform not found"})),
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(sqlx::Error::Database(e)) if e.code().as_deref() == Some("23503") => {
            HttpResponse::Conflict().json(serde_json::json!({"status": "fail","message": "Platform still has data that references it"}))
        }
        Err(e) => {
            eprintln!("error deleting platform: {e:?}");
            HttpResponse::InternalServerError()
                .json(json!({"status": "error", "message": "Internal error"}))
        }
    }
}
pub fn config(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/api")
        .service(health_checker_handler)
        .service(game_list_handler)
        .service(create_game_handler)
        .service(get_game_handler)
        .service(edit_game_handler)
        .service(delete_game_handler)
        .service(create_platform_handler)
        .service(get_platform_handler)
        .service(edit_platform_handler)
        .service(delete_platform_handler)
        .service(platform_list_handler);

    conf.service(scope);
}