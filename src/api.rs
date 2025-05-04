use crate::core::{SearchSystems, search_systems_read};
use poem::{error::InternalServerError, web::Data};
use poem_openapi::{OpenApi, Tags, param::Query, payload::Json};
use sqlx::{PgPool, Postgres, Transaction};

#[derive(Tags)]
enum Tag {
    Search,
    System,
    #[oai(rename = "Dag Run")]
    DagRun,
    Log,
}

/// Struct we will use to build our REST API
pub struct Api;

#[OpenApi]
impl Api {
    /// Serach for your system
    #[oai(path = "/search_systems", method = "get", tag = Tag::Search)]
    async fn search_systems_get(
        &self,
        Data(pool): Data<&PgPool>,
        Query(search_by): Query<String>,
        Query(page): Query<u32>,
    ) -> Result<Json<SearchSystems>, poem::Error> {
        // Start Transaction
        let mut tx: Transaction<'_, Postgres> = pool.begin().await.map_err(InternalServerError)?;

        // Search for anything that meets our criteria
        let systems: SearchSystems = search_systems_read(&mut tx, &search_by, &page).await?;

        Ok(Json(systems))
    }
}
