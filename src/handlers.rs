use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{Path, Query},
    http::{HeaderMap, StatusCode, Uri},
    Extension, Json,
};
use sqlx::SqlitePool;

use crate::{
    authentication::{AuthUser, MaybeUser},
    data_formats::LoginRequest,
    db_helpers::{
        follow_user_in_db, get_profile_from_db, get_user_by_id, unfollow_user_in_db,
        update_user_in_db,
    },
    errors::{RequestError, RequestErrorJsonWrapper},
    ArticleWrapper, RegisterRequest, UserResponse, UserWrapper,
};

use crate::authentication::{get_jwt_token, hash_password_argon2, verify_password_argon2};

use crate::{ProfileResponse, ProfileWrapper, UpdateUserRequest};

type UserJson = UserWrapper<UserResponse>;
type ProfileJson = ProfileWrapper;

type JsonResult<T> = Result<Json<T>, (StatusCode, Json<RequestErrorJsonWrapper>)>;

// ----------------- Helper Handlers -----------------
pub async fn alive() -> &'static str {
    "alive"
}

pub async fn not_found(uri: Uri) -> Result<(), (StatusCode, String)> {
    Err((
        StatusCode::NOT_FOUND,
        format!("URL {} provided was not found", uri),
    ))
}

// ----------------- User Handlers -----------------
pub async fn login_user(
    Extension(pool): Extension<Arc<SqlitePool>>,
    Json(UserWrapper { user: request }): Json<UserWrapper<LoginRequest>>,
) -> JsonResult<UserJson> {
    let user = crate::db_helpers::get_user_by_email(&pool, &request.email)
        .await
        .map_err(|_| {
            RequestError::RunTimeError("Could not login user\nPlease Try again").to_json_response()
        })?;
    let user = match user {
        Some(user) => user,
        None => {
            return Err(RequestError::RunTimeError("Email not found").to_json_response());
        }
    };
    let is_password_correct = verify_password_argon2(request.password, user.password)
        .await
        .map_err(|_| {
            RequestError::RunTimeError("Could not login user\nPlease Try again").to_json_response()
        })?;

    if !is_password_correct {
        return Err(RequestError::RunTimeError("Incorrect password").to_json_response());
    }
    let token = get_jwt_token(user.id).unwrap();
    let result = UserResponse {
        email: user.email,
        token,
        username: user.username,
        bio: user.bio.unwrap_or(String::new()),
        image: user.image,
    };
    Ok(Json(UserWrapper::wrap_with_user_data(result)))
}

pub async fn register_user(
    Extension(pool): Extension<Arc<SqlitePool>>,
    Json(UserWrapper { mut user }): Json<UserWrapper<RegisterRequest>>,
) -> JsonResult<UserJson> {
    user.password = hash_password_argon2(user.password).await.map_err(|_| {
        RequestError::RunTimeError("Could not register user\nPlease Try: again").to_json_response()
    })?;

    let user_id = crate::db_helpers::insert_user(&pool, &user)
        .await
        .map_err(|e| {
            if let RequestError::DatabaseError(sqlx::Error::Database(e)) = e {
                if e.message().contains("UNIQUE constraint failed") {
                    return RequestError::RunTimeError("Email already exists").to_json_response();
                }
            }
            RequestError::RunTimeError("Could not register user").to_json_response()
        })?;

    let token = get_jwt_token(user_id).map_err(|_| {
        RequestError::RunTimeError("Could not generate JWT successfully\nTry again later")
            .to_json_response()
    })?;
    let result = UserResponse {
        email: user.email,
        token,
        username: user.username,
        bio: String::new(),
        image: None,
    };
    Ok(Json(UserWrapper::wrap_with_user_data(result)))
}

pub async fn get_current_user(
    Extension(pool): Extension<Arc<SqlitePool>>,
    MaybeUser(maybe_user): MaybeUser,
) -> JsonResult<UserJson> {
    if let Some(AuthUser { id, token }) = maybe_user {
        let user = get_user_by_id(&pool, id)
            .await
            .map_err(|_| RequestError::ServerError.to_json_response())?;
        let user = match user {
            Some(user) => user,
            None => {
                return Err(RequestError::RunTimeError("User not found").to_json_response());
            }
        };
        let result = UserResponse {
            email: user.email,
            token,
            username: user.username,
            bio: user.bio.unwrap_or(String::new()),
            image: user.image,
        };
        return Ok(Json(UserWrapper::wrap_with_user_data(result)));
    }
    Err(RequestError::NotAuthorized("Need to be authorized").to_json_response())
}

// TODO: Add check if the DB error was whether or not the user exists
pub async fn update_user(
    MaybeUser(maybe_user): MaybeUser,
    Extension(pool): Extension<Arc<SqlitePool>>,
    Json(UserWrapper { user }): Json<UserWrapper<UpdateUserRequest>>,
) -> JsonResult<UserJson> {
    if let Some(AuthUser { id, token }) = maybe_user {
        let user = update_user_in_db(&pool, id, user)
            .await
            //? Add TODO Fix here
            .map_err(|_| RequestError::ServerError.to_json_response())?;
        let result = UserResponse {
            email: user.email,
            token,
            username: user.username,
            bio: user.bio.unwrap_or(String::new()),
            image: user.image,
        };
        return Ok(Json(UserWrapper::wrap_with_user_data(result)));
    }
    Err(RequestError::Forbidden.to_json_response())
}
// ----------------- End User Handlers -----------------

// ----------------- Profile Handlers -----------------
pub async fn get_profile(
    Extension(pool): Extension<Arc<SqlitePool>>,
    maybe_user: MaybeUser,
    Path(username): Path<String>,
) -> JsonResult<ProfileJson> {
    let (profile, following) = get_profile_from_db(&pool, maybe_user.get_id(), &username)
        .await
        .map_err(|e| e.to_json_response())?;
    let result = ProfileResponse {
        username,
        bio: profile.bio.unwrap_or_default(),
        image: profile.image,
        following,
    };
    Ok(Json(ProfileWrapper { profile: result }))
}

pub async fn follow_profile(
    MaybeUser(maybe_user): MaybeUser,
    Extension(pool): Extension<Arc<SqlitePool>>,
    Path(username): Path<String>,
) -> JsonResult<ProfileJson> {
    if let Some(user) = maybe_user {
        let profile = follow_user_in_db(&pool, user.id, &username)
            .await
            .map_err(|e| {
                if let RequestError::DatabaseError(sqlx::Error::Database(e)) = e {
                    if e.message().contains("UNIQUE constraint failed") {
                        return RequestError::RunTimeError("User already follows the other user")
                            .to_json_response();
                    }
                }
                RequestError::ServerError.to_json_response()
            })?;
        let result = ProfileResponse {
            username,
            bio: profile.bio.unwrap_or_default(),
            image: profile.image,
            following: true,
        };
        return Ok(Json(ProfileWrapper { profile: result }));
    }
    Err(RequestError::NotAuthorized("Need to be authorized").to_json_response())
}

pub async fn unfollow_profile(
    MaybeUser(user): MaybeUser,
    Extension(pool): Extension<Arc<SqlitePool>>,
    Path(username): Path<String>,
) -> JsonResult<ProfileJson> {
    if let Some(user) = user {
        let profile = unfollow_user_in_db(&pool, user.id, &username)
            .await
            .map_err(|e| e.to_json_response())?;
        let result = ProfileResponse {
            username,
            bio: profile.bio.unwrap_or_default(),
            image: profile.image,
            following: true,
        };
        return Ok(Json(ProfileWrapper { profile: result }));
    }
    Err(RequestError::NotAuthorized("Need to be authorized").to_json_response())
}

// ----------------- Article Handlers -----------------

pub async fn list_articles(Query(params): Query<HashMap<String, String>>) -> Json<ArticleWrapper> {
    todo!()
}

// ----------------- Comment Handlers -----------------

pub async fn get_comment(Query(params): Query<HashMap<String, String>>) -> &'static str {
    todo!()
}
