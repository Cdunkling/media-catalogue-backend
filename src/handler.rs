use crate::{
    AppState,
    schema::UserOut,
    schema::FilterOptions,
    model::GameModel,
    model::UserModel,
    model::PlatformModel,
    schema::{GameInput, GamePatch, PasswordChange, PlatformInput, UserInput, UserPatch}
};
use actix_web::{delete, get, patch, post, web, HttpResponse, Responder};
use serde_json::json;
use argon2::{password_hash::PasswordHasher, password_hash::PasswordVerifier, Argon2, PasswordHash};
use password_hash::phc::SaltString;

#[get("/healthchecker")]
async fn health_checker_handler() -> impl Responder {
    const MESSAGE: &str = "Build API with Rust, SQLX, Postgres,and Actix Web";

    HttpResponse::Ok().json(json!({"status": "success","message": MESSAGE}))
}

#[get("/users")]
pub async fn user_list_handler(
    opts: web::Query<FilterOptions>,
    data: web::Data<AppState>,
) -> impl Responder {
    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    let query_result = sqlx::query_as!(
        UserModel,
        "SELECT * FROM users ORDER by User_ID LIMIT $1 OFFSET $2",
        limit as i32,
        offset as i32
    )
    .fetch_all(&data.db)
    .await;

    if query_result.is_err() {
        let message = "Something bad happened while fetching all user items";
        return HttpResponse::InternalServerError()
            .json(json!({"status": "error","message": message}));
    }

    let users = query_result.unwrap();

    let json_response = serde_json::json!({
        "status": "success",
        "results": users.len(),
        "users": users
    });
    HttpResponse::Ok().json(json_response)
}

#[post("/users")]
async fn create_user_handler(
    body: web::Json<UserInput>,
    data: web::Data<AppState>,
) -> impl Responder {
    if !(3..=50).contains(&body.username.len()) {
        return HttpResponse::BadRequest()
            .json(json!({"status": "fail", "message": "Username must be 3-50 characters"}));
    }
    if body.password.len() < 8 {
        return HttpResponse::BadRequest()
            .json(json!({"status": "fail", "message": "Password must be at least 8 characters"}));
    }

    let salt = SaltString::generate();
    let new_hash = match Argon2::default().hash_password(body.password.as_bytes()) {
        Ok(h) => h.to_string(),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(json!({"status": "error", "message": "Internal error"}))
        }
    };
    let result = sqlx::query_as!(
        UserOut,
        r#"INSERT INTO users (username, password_hash, email)
           VALUES ($1, $2, $3)
           RETURNING user_id, username, email, join_date, last_online"#,
        body.username,
        new_hash,
        body.email
    )
        .fetch_one(&data.db)
        .await;

    match result {
        Ok(user) => HttpResponse::Created()
            .json(json!({"status": "success", "data": {"user": user}})),
        Err(sqlx::Error::Database(e)) if e.code().as_deref() == Some("23505") =>
            HttpResponse::Conflict()
                .json(json!({"status": "fail", "message": "Username or Email already exists"})),
        Err(e) => {
            eprintln!("failed to create user: {e}"); // or tracing::error!
            HttpResponse::InternalServerError()
                .json(json!({"status": "error", "message": "failed to create user"}))
        }
    }
}

/// Retrieves a single user by their ID.
///
/// This handler fetches a user from the database using the provided user ID from the URL path.
/// If the user is found, it returns a JSON response with the user data and a success status.
/// If the user is not found, it returns a 404 Not Found response with an error message.
///
/// # Parameters
///
/// * `path` - A `web::Path<i32>` containing the user ID extracted from the URL path parameter.
/// * `data` - A `web::Data<AppState>` containing the shared application state, including the database connection pool.
///
/// # Returns
///
/// Returns an `impl Responder` which will be one of:
/// * `HttpResponse::Ok` with JSON containing the user data if the user is found.
/// * `HttpResponse::NotFound` with JSON containing an error message if the user does not exist.
#[get("/users/{id}")]
async fn get_user_handler(
    path: web::Path<i32>,
    data: web::Data<AppState>,
) -> impl Responder {
    let user_id = path.into_inner();
    let query_result = sqlx::query_as!(
        UserModel,
        "SELECT * FROM users WHERE User_ID = $1",
        user_id
    )
    .fetch_one(&data.db)
    .await;

    match query_result {
        Ok(user) => {
            let user_response = serde_json::json!({"status": "success","data": serde_json::json!({
                "user": user
            })});

            return HttpResponse::Ok().json(user_response);
        }
        Err(_) => {
            let message = format!("user with ID: {} not found", user_id);
            return HttpResponse::NotFound()
                .json(serde_json::json!({"status": "fail","message": message}));
        }
    }
}

#[post("/users/{id}/password")]
async fn edit_pass_handler(
    path: web::Path<i32>,
    body: web::Json<PasswordChange>,
    data: web::Data<AppState>,
) -> impl Responder {
    let user_id = path.into_inner();
    let stored = match sqlx::query_scalar!(
        "SELECT password_hash FROM users WHERE user_id = $1",
        user_id
    )
    .fetch_optional(&data.db)
    .await
    {
        Ok(Some(h)) => h,
        Ok(None) => {
        return HttpResponse::NotFound()
            .json(serde_json::json!({"status": "fail", "message": "User not found"}))
        }
        Err(e) => {
        eprintln!("password lookup error: {e:?}");
        return HttpResponse::InternalServerError()
            .json(serde_json::json!({"status": "error", "message": "Internal error"}));
        }
    };

    let argon2 = Argon2::default();

    let verified = PasswordHash::new(&stored)
        .map(|parsed| argon2.verify_password(body.current_password.as_bytes(), &parsed).is_ok())
        .unwrap_or(false);
    if !verified {
        return HttpResponse::Unauthorized()
            .json(serde_json::json!({"status": "fail", "message": "Current password is incorrect"}));
    }

    if body.new_password.len() < 8 {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"status": "fail", "message": "Password must be at least 8 characters"}));
    }

    let salt = SaltString::generate();
    let new_hash = match argon2.hash_password(body.new_password.as_bytes()) {
        Ok(h) => h.to_string(),
        Err(_) => {
            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "error", "message": "Internal error"}))
        }
    };
    match sqlx::query!(
        "UPDATE users SET password_hash = $1, updated_at = now() WHERE user_id = $2",
        new_hash,
        user_id
    )
        .execute(&data.db)
        .await
    {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"status": "success"})),
        Err(e) => {
            eprintln!("password lookup error: {e:?}");
            return HttpResponse::InternalServerError()
            .json(serde_json::json!({"status": "error", "message": "Internal error"}));
        }
    }
}

#[patch("/users/{id}")]
async fn edit_user_handler(
    path: web::Path<i32>,
    body: web::Json<UserPatch>,
    data: web::Data<AppState>,
) -> impl Responder {
    let user_id = path.into_inner();
    let result = sqlx::query_as!(
        UserOut,
        r#"UPDATE users SET
               username   = COALESCE($1, username),
               email      = COALESCE($2, email),
               updated_at = now()
           WHERE user_id = $3
           RETURNING user_id, username, email, join_date, last_online"#,
        body.username,
        body.email,
        user_id,
    )
    .fetch_optional(&data.db)
    .await;

    match result {
        Ok(Some(user)) => HttpResponse::Ok()
            .json(serde_json::json!({"status": "success", "data": {"user": user}})),
        Ok(None) => HttpResponse::NotFound()
            .json(serde_json::json!({"status": "fail", "message": "User not found"})),
        Err(sqlx::Error::Database(e)) if e.code().as_deref() == Some("23505") => {
            HttpResponse::Conflict()
            .json(serde_json::json!({"status": "error", "message": "User already exists"}))
        }
        Err(e) => {

            eprintln!("password lookup error: {e}");
            HttpResponse::InternalServerError()
            .json(serde_json::json!({"status": "error", "message": "Internal error"}))
        }
    }

}

#[delete("/users/{id}")]
async fn delete_user_handler(
    path: web::Path<i32>,
    data: web::Data<AppState>,
) -> impl Responder {
    let user_id = path.into_inner();

    match sqlx::query!("DELETE FROM users WHERE user_id = $1", user_id)
        .execute(&data.db)
        .await
    {
        Ok(res) if res.rows_affected() == 0 => HttpResponse::NotFound()
            .json(json!({"status": "fail", "message": "User not found"})),
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(sqlx::Error::Database(e)) if e.code().as_deref() == Some("23503") => {
            HttpResponse::Conflict().json(json!({
                "status": "fail",
                "message": "User still has data that references it"
            }))
        }
        Err(e) => {
            eprintln!("delete_user error: {e:?}");
            HttpResponse::InternalServerError()
                .json(json!({"status": "error", "message": "Internal error"}))
        }
    }
}

// game Handlers


#[get("/games")]
pub async fn game_list_handler(
    opts: web::Query<FilterOptions>,
    data: web::Data<AppState>,
) -> impl Responder {
    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    let result = sqlx::query_as!(
        GameModel,
        "SELECT * FROM games ORDER by id LIMIT $1 OFFSET $2",
        limit as i32,
        offset as i32
    )
        .fetch_all(&data.db)
        .await;

    match result {
        Ok(games) => HttpResponse::Ok().json(serde_json::json!({
            "status": "success",
            "results": games.len(),
            "page": offset,
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
    if (body.title.len()==0) {
        return HttpResponse::BadRequest()
            .json(json!({"status": "fail", "message": "Title must not be blank"}));
    }
    let query_result = sqlx::query_as!(
    GameModel,
    r#"INSERT INTO games (
        title, description, genre, price_paid, barcode, region, images,
        year_released, developer, publisher, rating, status, hours_played,
        date_added, last_updated, platform_id
    )
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13,
            CURRENT_TIMESTAMP, CURRENT_TIMESTAMP,$14)
    RETURNING *"#,
    body.title,
    body.description,
    body.genre,
    body.price_paid,
    body.barcode,
    body.region,
    body.images,
    body.year_released,
    body.developer,
    body.publisher,
    body.rating,
    body.status,
    body.hours_played,
    body.platform_id,
)
        .fetch_one(&data.db)
        .await;

    match query_result {
        Ok(game) => {
            let game_response = serde_json::json!({"status": "success","data": serde_json::json!({
                "game": game
            })});

            return HttpResponse::Ok().json(game_response);
        }
        Err(e) => {
            if e.to_string()
                .contains("duplicate key value violates unique constraint")
            {
                return HttpResponse::BadRequest()
                    .json(serde_json::json!({"status": "fail","message": "game with that title already exists"}));
            }

            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "error","message": format!("{:?}", e)}));
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
async fn get_game_handler(
    path: web::Path<String>,
    data: web::Data<AppState>,
) -> impl Responder {
    let game_id = path.into_inner();
    let query_result = sqlx::query_as!(
        GameModel,
        "SELECT * FROM games WHERE ID = $1",
        game_id.parse::<i32>().unwrap()
    )
    .fetch_one(&data.db)
    .await;

    match query_result {
        Ok(game) => {
            let game_response = serde_json::json!({"status": "success","data": serde_json::json!({
                "game": game
            })});

            return HttpResponse::Ok().json(game_response);
        }
        Err(_) => {
            let message = format!("game with ID: {} not found", game_id);
            return HttpResponse::NotFound()
                .json(serde_json::json!({"status": "fail","message": message}));
        }
    }
}

#[patch("/games/{id}")]
async fn edit_game_handler(
    path: web::Path<i32>,
    body: web::Json<GamePatch>,
    data: web::Data<AppState>,
) -> impl Responder {
    let game_id = path.into_inner();
    let result = sqlx::query_as!(
        GameModel,
        r#"UPDATE games SET
                 title=coalesce($1,title),
                 description=coalesce($2,description),
                 genre=coalesce($3,genre),
                 price_paid=coalesce($4,price_paid),
                 barcode=coalesce($5,barcode),
                 region=coalesce($6,region),
                 year_released=coalesce($7,year_released),
                 last_updated=now()
             WHERE ID = $8
             RETURNING *"#,
        body.title.to_owned(),
        body.description.to_owned(),
        body.genre.to_owned(),
        body.price_paid.to_owned(),
        body.barcode.to_owned(),
        body.region.to_owned(),
        body.year_released.to_owned(),
        game_id,
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
        Err(e) => {
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
            .json(serde_json::json!({"status": "fail", "message": "User not found"})),
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
    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;

    let query_result = sqlx::query_as!(
        PlatformModel,
        "SELECT * FROM platforms ORDER by platform_id LIMIT $1 OFFSET $2",
        limit as i32,
        offset as i32
    )
        .fetch_all(&data.db)
        .await;

    if query_result.is_err() {
        let message = "Something bad happened while fetching all platform items";
        return HttpResponse::InternalServerError()
            .json(json!({"status": "error","message": message}));
    }

    let platforms = query_result.unwrap();

    let json_response = serde_json::json!({
        "status": "success",
        "results": platforms.len(),
        "platforms": platforms
    });
    HttpResponse::Ok().json(json_response)
}
#[post("/platforms")]
async fn create_platform_handler(
    body: web::Json<PlatformInput>,
    data: web::Data<AppState>,
) -> impl Responder {
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
        Ok(platform) => {
            let platform_response = serde_json::json!({"status": "success","data": serde_json::json!({
                "platform": platform
            })});

            return HttpResponse::Ok().json(platform_response);
        }
        Err(e) => {
            if e.to_string()
                .contains("duplicate key value violates unique constraint")
            {
                return HttpResponse::BadRequest()
                    .json(serde_json::json!({"status": "fail","message": "platform with that title already exists"}));
            }

            return HttpResponse::InternalServerError()
                .json(serde_json::json!({"status": "error","message": format!("{:?}", e)}));
        }
    }
}

pub fn config(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/api")
        .service(health_checker_handler)
        .service(user_list_handler)
        .service(create_user_handler)
        .service(get_user_handler)
        .service(edit_user_handler)
        .service(delete_user_handler)
        .service(game_list_handler)
        .service(create_game_handler)
        .service(get_game_handler)
        .service(edit_game_handler)
        .service(delete_game_handler)
        .service(create_platform_handler)
        .service(platform_list_handler);

    conf.service(scope);
}