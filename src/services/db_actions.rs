#![allow(unused)]

use crate::models::api_models::CrosswordMetadata;
use crate::models::db_models::{Crossword, InsertableCrossword};
use crate::models::errors::AppError;
use crate::models::errors::AppError::InternalServerError;
use crate::models::guardian::GuardianCrossword;
use crate::schema::crossword::dsl::{crossword, crossword_json, date, id, series};
use crate::DbPool;
use actix_web::web;
use diesel::row::NamedRow;
use diesel::{ExpressionMethods, QueryDsl, QueryResult, RunQueryDsl};
use serde_json::Value;

pub async fn get_crossword_ids_for_series(
    pool: web::Data<DbPool>,
    series_for: String,
) -> actix_web::Result<Vec<String>, AppError> {
    // use web::block to offload blocking Diesel queries without blocking server thread
    web::block(move || {
        let mut conn = pool.get()?;
        crossword
            .filter(series.eq(series_for))
            .select(id)
            .load(&mut conn)
            .map_err(|e| AppError::InternalServerError(e.to_string()))
    })
    .await?
}

pub async fn get_crossword_metadata_for_series(
    pool: web::Data<DbPool>,
    series_for: String,
) -> actix_web::Result<Vec<CrosswordMetadata>, AppError> {
    // use web::block to offload blocking Diesel queries without blocking server thread
    web::block(move || {
        let mut conn = pool.get()?;
        crossword
            .filter(series.eq(series_for))
            .select((id, series, date))
            .load(&mut conn)
            .map_err(|e| AppError::InternalServerError(e.to_string()))
    })
    .await?
}

pub async fn get_crossword_for_series_and_id(
    pool: web::Data<DbPool>,
    id_for: String,
    series_for: String,
) -> actix_web::Result<GuardianCrossword, AppError> {
    // use web::block to offload blocking Diesel queries without blocking server thread
    let result: Value = web::block(move || {
        let mut conn = pool.get()?;
        crossword
            .filter(id.eq(id_for.clone()))
            .filter(series.eq(series_for))
            .select(crossword_json)
            .first(&mut conn)
            .map_err(|_| AppError::CrosswordNotFound(id_for.clone()))
    })
    .await??;
    serde_json::from_value(result).map_err(|e| AppError::InternalServerError(e.to_string()))
}

pub async fn store_crosswords(
    pool: web::Data<DbPool>,
    crosswords: Vec<Crossword>,
) -> actix_web::Result<usize, AppError> {
    // use web::block to offload blocking Diesel queries without blocking server thread1
    web::block(move || {
        let mut conn = pool.get()?;
        let insertable_crosswords: Vec<InsertableCrossword> =
            crosswords.iter().map(crossword_to_insertable).collect();
        diesel::insert_into(crossword)
            .values(insertable_crosswords)
            .execute(&mut conn)
            .map_err(|e| AppError::InternalServerError(e.to_string()))
    })
    .await?
}

fn crossword_to_insertable(crossword_to_insert: &Crossword) -> InsertableCrossword {
    InsertableCrossword {
        id: crossword_to_insert.id.as_str(),
        series: crossword_to_insert.series.as_str(),
        date: crossword_to_insert.date,
        crossword_json: crossword_to_insert.crossword_json.clone(),
    }
}