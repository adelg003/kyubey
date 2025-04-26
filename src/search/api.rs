use crate::{
    AppState,
    search::core::{SearchSystems, search_systems_read},
    util::sqlx_to_axum_error,
};
use axum::{
    extract::{Path, Query, Request, State},
    http::{header::ACCEPT, status::StatusCode},
    response::{Html, IntoResponse, Json, Response},
};
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

/// What content-types wes should accept
enum Accept {
    Html,
    Json,
}

impl Accept {
    // Create Accept from a string
    fn from_str(str: &str) -> Option<Accept> {
        match str {
            "text/html" => Some(Accept::Html),
            "application/json" => Some(Accept::Json),
            _ => None,
        }
    }

    // Pull the "Accepted" type from a request
    fn from_request(request: &Request) -> Option<Accept> {
        // Dump the full body of the accept header. The accept header can be a comma seperate list.
        let accept_body: &str = request.headers().get(ACCEPT)?.to_str().ok()?;

        // In the accept headers body, do we have the expected value?
        accept_body
            .split(",")
            .flat_map(|accept: &str| Accept::from_str(accept))
            .next()
    }
}

/// Test response scruct
#[derive(Serialize, ToSchema)]
struct User {
    //TODO Remove test
    name: String,
}

/// Render the router for systems
pub fn router() -> OpenApiRouter<AppState> {
    OpenApiRouter::<AppState>::new().routes(routes!(hello_world, search_systems_get))
}

/// Test endpoint
#[utoipa::path(
    get,
    path = "/test/{name}",
    params(
        ("name" = String, Path),
    ),
    responses(
        (status = OK, content_type = "application/json", body = User),
    ),
    tag = "Test",
)]
async fn hello_world(Path(name): Path<String>, request: Request) -> Response {
    //TODO Remove test endpoint
    // Build the needed data scruct to respond
    let user: User = User { name };

    // Pull what formate the clients is expecting
    let accept: Option<Accept> = Accept::from_request(&request);

    match accept {
        Some(Accept::Html) => Html(format!("<h1>Hello {}</h1>", user.name)).into_response(),
        Some(Accept::Json) => Json(user).into_response(),
        None => (
            StatusCode::BAD_REQUEST,
            "Valid HTTP header \"Accept\" missing. Accept headers \"application/json\" or \"text/html\" are required.",
        ).into_response(),
    }
}

/// Search for a System
#[utoipa::path(
    get,
    path = "/api/search_systems",
    params(
        ("search_by" = Option<String>, Query),
        ("page" = Option<u32>, Query),
    ),
    responses(
        //(status = OK, content_type = "application/json", body = SearchSystems),
    ),
    tag = "Search",
)]
async fn search_systems_get(
    Query(search_by): Query<Option<String>>,
    Query(page): Query<Option<u32>>,
    State(state): State<AppState>,
    request: Request,
) -> Result<Response, StatusCode> {
    // Default no page to 0
    let page: u32 = page.unwrap_or(0);

    // Start Transaction
    let mut tx = state.pool.begin().await.map_err(sqlx_to_axum_error)?;

    // Search for anything that meets our criteria
    let systems: SearchSystems = search_systems_read(&mut tx, &search_by, page).await?;

    // Pull what format the clients is expecting
    let accept: Option<Accept> = Accept::from_request(&request);

    // Determin how we want to respond
    match accept {
        Some(Accept::Html) => Ok(Html("<h1>Placeholder</h1>").into_response()),
        Some(Accept::Json) => Ok(Json(systems).into_response()),
        None => Err(StatusCode::BAD_REQUEST),
    }
}
