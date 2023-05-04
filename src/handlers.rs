use std::{collections::HashMap, sync::Arc};

use axum::{
    extract::{rejection::JsonRejection, Path, Query},
    http::{StatusCode, Uri},
    Extension, Json,
};
use sqlx::SqlitePool;

use crate::{
    authentication::{AuthUser, MaybeUser},
    data_formats::{request::*, response::*, wrapper::*, ArticleQueryParams},
    db_helpers::*,
    errors::RequestError,
};

use crate::authentication::{get_jwt_token, hash_password_argon2, verify_password_argon2};

// use crate::{ProfileResponse, ProfileWrapper, UpdateUserRequest};

type UserJson = UserWrapper<UserResponse>;
type ProfileJson = ProfileWrapper;
type ArticleJson = ArticleWrapper<ArticleResponse>;
type CommentJson = CommentWrapper<CommentResponse>;

type JsonResult<T> = Result<(StatusCode, Json<T>), RequestError>;

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
    payload: Result<Json<UserWrapper<LoginRequest>>, JsonRejection>,
) -> JsonResult<UserJson> {
    let Json(UserWrapper { user: request }): Json<UserWrapper<LoginRequest>> = payload?;
    let user = get_user_by_email(&pool, &request.email)
        .await
        .map_err(|_| RequestError::RunTimeError("Could not login user\nPlease Try again"))?;
    let user = match user {
        Some(user) => user,
        None => {
            return Err(RequestError::RunTimeError("Email not found"));
        }
    };
    let is_password_correct = verify_password_argon2(request.password, &user.password)
        .await
        .map_err(|_| RequestError::RunTimeError("Could not login user\nPlease Try again"))?;

    if !is_password_correct {
        return Err(RequestError::RunTimeError("Incorrect password"));
    }
    let token = get_jwt_token(user.id).unwrap();
    let result = UserResponse::new(user, token);
    Ok((
        StatusCode::OK,
        Json(UserWrapper::wrap_with_user_data(result)),
    ))
}

pub async fn register_user(
    Extension(pool): Extension<Arc<SqlitePool>>,
    payload: Result<Json<UserWrapper<RegisterRequest>>, JsonRejection>,
) -> JsonResult<UserJson> {
    let Json(UserWrapper { mut user }): Json<UserWrapper<RegisterRequest>> = payload?;
    user.password = hash_password_argon2(user.password)
        .await
        .map_err(|_| RequestError::RunTimeError("Could not register user\nPlease Try: again"))?;

    let user = insert_user(&pool, &user).await.map_err(|e| {
        if let RequestError::DatabaseError(sqlx::Error::Database(e)) = e {
            if e.message().contains("UNIQUE constraint failed") {
                return RequestError::RunTimeError("Email already exists");
            }
        }
        RequestError::RunTimeError("Could not register user")
    })?;

    let token = get_jwt_token(user.id).map_err(|_| {
        RequestError::RunTimeError("Could not generate JWT successfully\nTry again later")
    })?;
    let result = UserResponse::new(user, token);
    Ok((
        StatusCode::CREATED,
        Json(UserWrapper::wrap_with_user_data(result)),
    ))
}

pub async fn get_current_user(
    Extension(pool): Extension<Arc<SqlitePool>>,
    MaybeUser(maybe_user): MaybeUser,
) -> JsonResult<UserJson> {
    if let Some(AuthUser { id, token }) = maybe_user {
        let user = get_user_by_id(&pool, id)
            .await
            .map_err(|_| RequestError::ServerError)?;
        let user = match user {
            Some(user) => user,
            None => {
                return Err(RequestError::NotFound("User not found"));
            }
        };
        let result = UserResponse::new(user, token);
        return Ok((
            StatusCode::OK,
            Json(UserWrapper::wrap_with_user_data(result)),
        ));
    }
    Err(RequestError::NotAuthorized("Need to be authorized"))
}

// TODO: Add check if the DB error was whether or not the user exists
pub async fn update_user(
    MaybeUser(maybe_user): MaybeUser,
    Extension(pool): Extension<Arc<SqlitePool>>,
    payload: Result<Json<UserWrapper<UpdateUserRequest>>, JsonRejection>,
) -> JsonResult<UserJson> {
    let Json(UserWrapper { user }): Json<UserWrapper<UpdateUserRequest>> = payload?;
    if let Some(AuthUser { id, token }) = maybe_user {
        let user = update_user_in_db(&pool, id, user)
            .await
            //? Add TODO Fix here
            .map_err(|_| RequestError::ServerError)?;
        let result = UserResponse::new(user, token);
        return Ok((
            StatusCode::OK,
            Json(UserWrapper::wrap_with_user_data(result)),
        ));
    }
    Err(RequestError::NotAuthorized("Need to be authorized"))
}
// ----------------- End User Handlers -----------------

// ----------------- Profile Handlers -----------------
pub async fn get_profile(
    Extension(pool): Extension<Arc<SqlitePool>>,
    maybe_user: MaybeUser,
    Path(username): Path<String>,
) -> JsonResult<ProfileJson> {
    let (profile, following) =
        get_profile_by_username_in_db(&pool, maybe_user.get_id(), &username).await?;
    let result = ProfileResponse::new(profile, following);
    Ok((StatusCode::OK, Json(ProfileWrapper { profile: result })))
}

pub async fn get_all_profile_username(
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> JsonResult<Vec<String>> {
    let result = get_all_profile_username_in_db(&pool).await?;
    Ok((StatusCode::OK, Json(result)))
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
                        return RequestError::RunTimeError("User already follows the other user");
                    }
                }
                RequestError::ServerError
            })?;
        let result = ProfileResponse::new(profile, true);
        return Ok((StatusCode::OK, Json(ProfileWrapper { profile: result })));
    }
    Err(RequestError::NotAuthorized("Need to be authorized"))
}

pub async fn unfollow_profile(
    MaybeUser(user): MaybeUser,
    Extension(pool): Extension<Arc<SqlitePool>>,
    Path(username): Path<String>,
) -> JsonResult<ProfileJson> {
    if let Some(user) = user {
        let profile = unfollow_user_in_db(&pool, user.id, &username).await?;
        let result = ProfileResponse::new(profile, false);
        return Ok((StatusCode::OK, Json(ProfileWrapper { profile: result })));
    }
    Err(RequestError::NotAuthorized("Need to be authorized"))
}
// ----------------- End Profile Handlers -----------------

// ----------------- Article Handlers -----------------

// This is a method to allow the frontend(NextJS getStaticPath) to get all slugs
pub async fn get_all_slugs(Extension(pool): Extension<Arc<SqlitePool>>) -> JsonResult<Vec<String>> {
    let slugs = get_all_slugs_in_db(&pool).await?;
    Ok((StatusCode::OK, Json(slugs)))
}

pub async fn list_articles(
    Extension(pool): Extension<Arc<SqlitePool>>,
    maybe_user: MaybeUser,
    Query(params): Query<HashMap<String, String>>,
) -> JsonResult<MultipleArticlesWrapper> {
    let data: ArticleQueryParams = serde_json::from_value(serde_json::json!(params))
        .map_err(|_| RequestError::RunTimeError("Incorrect Query Paramaters"))?;
    let articles = list_all_articles(&pool, maybe_user.get_id(), data).await?;

    let articles = articles
        .into_iter()
        .map(ArticleResponse::new)
        .collect::<Vec<ArticleResponse>>();
    let article_count = articles.len();

    Ok((
        StatusCode::OK,
        Json(MultipleArticlesWrapper {
            articles,
            article_count,
        }),
    ))
}

pub async fn get_article(
    Extension(pool): Extension<Arc<SqlitePool>>,
    maybe_user: MaybeUser,
    Path(slug): Path<String>,
) -> JsonResult<ArticleJson> {
    let article = get_article_by_slug_in_db(&pool, &slug, maybe_user.get_id()).await?;
    let article = match article {
        Some(article) => article,
        None => {
            return Err(RequestError::NotFound("Article not found"));
        }
    };
    let article = ArticleResponse::new(article);
    Ok((StatusCode::OK, Json(ArticleWrapper { article })))
}

pub async fn create_article(
    MaybeUser(maybe_user): MaybeUser,
    Extension(pool): Extension<Arc<SqlitePool>>,
    Json(ArticleWrapper { article }): Json<ArticleWrapper<CreateArticleRequest>>,
) -> JsonResult<ArticleJson> {
    if let Some(user) = maybe_user {
        let article = create_article_in_db(&pool, user.id, article).await?;
        let article = ArticleResponse::new(article);
        return Ok((StatusCode::CREATED, Json(ArticleWrapper { article })));
    }
    Err(RequestError::NotAuthorized("Need to be authorized"))
}

pub async fn delete_article(
    Extension(pool): Extension<Arc<SqlitePool>>,
    maybe_user: MaybeUser,
    Path(slug): Path<String>,
) -> Result<StatusCode, RequestError> {
    if let Some(id) = maybe_user.get_id() {
        delete_article_in_db(&pool, id, &slug).await?;
        return Ok(StatusCode::NO_CONTENT);
    }
    Err(RequestError::NotAuthorized("Need to be authorized"))
}

pub async fn get_article_feed(
    Extension(pool): Extension<Arc<SqlitePool>>,
    MaybeUser(maybe_user): MaybeUser,
    Query(params): Query<HashMap<String, String>>,
) -> JsonResult<MultipleArticlesWrapper> {
    if let Some(user) = maybe_user {
        let data: ArticleQueryParams = serde_json::from_value(serde_json::json!(params))
            .map_err(|_| RequestError::RunTimeError("Could not parse query params"))?;
        let articles = list_articles_feed_in_db(&pool, user.id, data).await?;
        let articles = articles
            .into_iter()
            .map(ArticleResponse::new)
            .collect::<Vec<ArticleResponse>>();
        let article_count = articles.len();
        return Ok((
            StatusCode::OK,
            Json(MultipleArticlesWrapper {
                articles,
                article_count,
            }),
        ));
    }
    Err(RequestError::NotAuthorized("Need to be authorized"))
}

pub async fn update_article(
    Extension(pool): Extension<Arc<SqlitePool>>,
    maybe_user: MaybeUser,
    Path(slug): Path<String>,
    payload: Result<Json<ArticleWrapper<UpdateArticleRequest>>, JsonRejection>,
) -> JsonResult<ArticleJson> {
    let Json(ArticleWrapper { article }): Json<ArticleWrapper<UpdateArticleRequest>> = payload?;
    if let Some(id) = maybe_user.get_id() {
        let article = update_article_in_db(&pool, id, &slug, article).await?;
        let article = ArticleResponse::new(article);
        return Ok((StatusCode::OK, Json(ArticleWrapper { article })));
    }
    Err(RequestError::NotAuthorized("Need to be authorized"))
}

pub async fn favourite_article(
    Path(slug): Path<String>,
    MaybeUser(maybe_user): MaybeUser,
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> JsonResult<ArticleJson> {
    if let Some(user) = maybe_user {
        let article = favourite_article_in_db(&pool, &slug, user.id).await?;
        let article = ArticleResponse::new(article);
        return Ok((StatusCode::OK, Json(ArticleWrapper { article })));
    }
    Err(RequestError::NotAuthorized("Need to be authorized"))
}

pub async fn unfavourite_article(
    Path(slug): Path<String>,
    MaybeUser(maybe_user): MaybeUser,
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> JsonResult<ArticleJson> {
    if let Some(user) = maybe_user {
        let article = unfavourite_article_in_db(&pool, user.id, &slug).await?;
        let article = ArticleResponse::new(article);
        return Ok((StatusCode::OK, Json(ArticleWrapper { article })));
    }
    Err(RequestError::NotAuthorized("Need to be authorized"))
}

// ----------------- End Article Handlers -----------------

// ----------------- Comment Handlers -----------------

pub async fn get_comment(
    Path(slug): Path<String>,
    Path(id): Path<i64>,
    maybe_user: MaybeUser,
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> JsonResult<CommentJson> {
    let comment = get_comment_for_article_in_db(&pool, id, &slug).await?;
    // This unwrap should be safe
    let (user, following) =
        get_profile_by_id_in_db(&pool, maybe_user.get_id(), comment.author_id).await?;
    let profile_response = ProfileResponse::new(user, following);

    let comment = CommentResponse::new(comment, profile_response);

    // let response = CommentResponse::new(comment);
    Ok((StatusCode::OK, Json(CommentWrapper { comment })))
}

pub async fn get_comments(
    Path(slug): Path<String>,
    maybe_user: MaybeUser,
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> JsonResult<MultipleCommentsWrapper> {
    let comments = get_comments_for_article_in_db(&pool, &slug).await?;
    let mut result = Vec::with_capacity(comments.len());
    for comment in comments {
        let (user, following) =
            get_profile_by_id_in_db(&pool, maybe_user.get_id(), comment.author_id).await?;
        let profile_response = ProfileResponse::new(user, following);
        let comment = CommentResponse::new(comment, profile_response);
        result.push(comment);
    }
    Ok((
        StatusCode::OK,
        Json(MultipleCommentsWrapper { comments: result }),
    ))
}

pub async fn add_comment(
    Path(slug): Path<String>,
    MaybeUser(maybe_user): MaybeUser,
    Extension(pool): Extension<Arc<SqlitePool>>,
    payload: Result<Json<CommentWrapper<CommentRequest>>, JsonRejection>,
) -> JsonResult<CommentJson> {
    let Json(CommentWrapper { comment }): Json<CommentWrapper<CommentRequest>> = payload?;
    if let Some(user) = maybe_user {
        let comment = add_comments_to_article_in_db(&pool, user.id, &slug, comment).await?;
        let user = match get_user_by_id(&pool, comment.author_id).await? {
            Some(user) => user,
            None => {
                return Err(RequestError::NotFound("User not found"));
            }
        };
        let profile_response = ProfileResponse::new(user, false);
        let comment = CommentResponse::new(comment, profile_response);
        return Ok((StatusCode::CREATED, Json(CommentWrapper { comment })));
    }
    Err(RequestError::NotAuthorized("Need to be authorized"))
}

pub async fn delete_comment(
    Path(slug): Path<String>,
    Path(id): Path<i64>,
    MaybeUser(maybe_user): MaybeUser,
    Extension(pool): Extension<Arc<SqlitePool>>,
) -> Result<StatusCode, RequestError> {
    if let Some(user) = maybe_user {
        delete_comment_in_db(&pool, user.id, id, &slug).await?;
        return Ok(StatusCode::NO_CONTENT);
    }
    Err(RequestError::NotAuthorized("Need to be authorized"))
}

// ----------------- End Comment Handlers -----------------

// ----------------- Tag Handlers -----------------
pub async fn get_tags(Extension(pool): Extension<Arc<SqlitePool>>) -> JsonResult<Tags> {
    let tag_list = get_tags_in_db(&pool).await?;
    Ok((StatusCode::OK, Json(Tags { tags: tag_list })))
}
