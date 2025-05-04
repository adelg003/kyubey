use crate::core::{SearchSystems, System, search_systems_read, system_read};
use poem::{error::InternalServerError, web::Data};
use poem_openapi::{
    OpenApi, Tags,
    param::{Path, Query},
    payload::Json,
};
use sqlx::{PgPool, Postgres, Transaction};

#[derive(Tags)]
enum Tag {
    #[oai(rename = "Dag Run")]
    DagRun,
    Log,
    Search,
    System,
}

/// Struct we will use to build our REST API
pub struct Api;

#[OpenApi]
impl Api {
    /// Pull a single system
    #[oai(path = "/system/:system_id", method = "get", tag = Tag::System)]
    async fn system_get(
        &self,
        Data(pool): Data<&PgPool>,
        Path(system_id): Path<String>,
    ) -> Result<Json<System>, poem::Error> {
        // Start Transaction
        let mut tx: Transaction<'_, Postgres> = pool.begin().await.map_err(InternalServerError)?;

        // Search for anything that meets our criteria
        let system: System = system_read(&mut tx, &system_id).await?;

        Ok(Json(system))
    }

    /// Search for your system
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
