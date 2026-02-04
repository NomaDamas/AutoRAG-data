use std::collections::HashMap;
use tauri::State;

use crate::db::{
    AddEvidenceRequest, CreateQueryRequest, EvidenceGroup, EvidenceItem, ImageChunkInfo, PageInfo,
    Query, QueryWithEvidence, RetrievalRelation, UpdateQueryRequest,
};
use crate::error::{AppError, Result};
use crate::state::AppState;

#[tauri::command]
pub async fn create_query(
    request: CreateQueryRequest,
    state: State<'_, AppState>,
) -> Result<QueryWithEvidence> {
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    // Insert the query
    let query = sqlx::query_as::<_, Query>(
        r#"
        INSERT INTO query (contents, query_to_llm, generation_gt)
        VALUES ($1, $2, $3)
        RETURNING id, contents, query_to_llm, generation_gt
        "#,
    )
    .bind(&request.contents)
    .bind(&request.query_to_llm)
    .bind(&request.generation_gt)
    .fetch_one(&pool)
    .await?;

    // Insert evidence relations for each group
    let mut evidence_groups = Vec::new();
    for (group_index, chunk_ids) in request.evidence_groups.iter().enumerate() {
        let mut items = Vec::new();
        for (group_order, chunk_id) in chunk_ids.iter().enumerate() {
            let relation = sqlx::query_as::<_, RetrievalRelation>(
                r#"
                INSERT INTO retrieval_relation (query_id, group_index, group_order, image_chunk_id)
                VALUES ($1, $2, $3, $4)
                RETURNING query_id, group_index, group_order, chunk_id, image_chunk_id
                "#,
            )
            .bind(query.id)
            .bind(group_index as i32)
            .bind(group_order as i32)
            .bind(chunk_id)
            .fetch_one(&pool)
            .await?;

            // Fetch chunk and page info
            let chunk = sqlx::query_as::<_, ImageChunkInfo>(
                r#"
                SELECT id, parent_page, mimetype
                FROM image_chunk
                WHERE id = $1
                "#,
            )
            .bind(chunk_id)
            .fetch_optional(&pool)
            .await?;

            let page = if let Some(ref c) = chunk {
                if let Some(parent_page) = c.parent_page {
                    sqlx::query_as::<_, PageInfo>(
                        r#"
                        SELECT id, page_num, document_id, mimetype, page_metadata
                        FROM page
                        WHERE id = $1
                        "#,
                    )
                    .bind(parent_page)
                    .fetch_optional(&pool)
                    .await?
                } else {
                    None
                }
            } else {
                None
            };

            items.push(EvidenceItem {
                relation,
                chunk,
                page,
            });
        }

        evidence_groups.push(EvidenceGroup {
            group_index: group_index as i32,
            items,
        });
    }

    Ok(QueryWithEvidence {
        query,
        evidence_groups,
    })
}

#[tauri::command]
pub async fn update_query(
    request: UpdateQueryRequest,
    state: State<'_, AppState>,
) -> Result<Query> {
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    let existing = sqlx::query_as::<_, Query>(
        r#"
        SELECT id, contents, query_to_llm, generation_gt
        FROM query
        WHERE id = $1
        "#,
    )
    .bind(request.id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Query {} not found", request.id)))?;

    let contents = request.contents.unwrap_or(existing.contents);
    let query_to_llm = request.query_to_llm.or(existing.query_to_llm);
    let generation_gt = request.generation_gt.or(existing.generation_gt);

    let query = sqlx::query_as::<_, Query>(
        r#"
        UPDATE query
        SET contents = $2, query_to_llm = $3, generation_gt = $4
        WHERE id = $1
        RETURNING id, contents, query_to_llm, generation_gt
        "#,
    )
    .bind(request.id)
    .bind(contents)
    .bind(query_to_llm)
    .bind(generation_gt)
    .fetch_one(&pool)
    .await?;

    Ok(query)
}

#[tauri::command]
pub async fn delete_query(query_id: i64, state: State<'_, AppState>) -> Result<bool> {
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    // Delete relations first (composite PK, no cascade assumed)
    sqlx::query("DELETE FROM retrieval_relation WHERE query_id = $1")
        .bind(query_id)
        .execute(&pool)
        .await?;

    sqlx::query("DELETE FROM query WHERE id = $1")
        .bind(query_id)
        .execute(&pool)
        .await?;

    Ok(true)
}

#[tauri::command]
pub async fn list_queries(state: State<'_, AppState>) -> Result<Vec<Query>> {
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    let queries = sqlx::query_as::<_, Query>(
        r#"
        SELECT id, contents, query_to_llm, generation_gt
        FROM query
        ORDER BY id DESC
        "#,
    )
    .fetch_all(&pool)
    .await?;

    Ok(queries)
}

#[tauri::command]
pub async fn get_query_with_evidence(
    query_id: i64,
    state: State<'_, AppState>,
) -> Result<QueryWithEvidence> {
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    let query = sqlx::query_as::<_, Query>(
        r#"
        SELECT id, contents, query_to_llm, generation_gt
        FROM query
        WHERE id = $1
        "#,
    )
    .bind(query_id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("Query {} not found", query_id)))?;

    // Fetch all relations ordered by group_index, then group_order
    let relations = sqlx::query_as::<_, RetrievalRelation>(
        r#"
        SELECT query_id, group_index, group_order, chunk_id, image_chunk_id
        FROM retrieval_relation
        WHERE query_id = $1
        ORDER BY group_index ASC, group_order ASC
        "#,
    )
    .bind(query_id)
    .fetch_all(&pool)
    .await?;

    // Group relations by group_index
    let mut groups_map: HashMap<i32, Vec<EvidenceItem>> = HashMap::new();
    for relation in relations {
        // Fetch chunk info if image_chunk_id is present
        let chunk = if let Some(chunk_id) = relation.image_chunk_id {
            sqlx::query_as::<_, ImageChunkInfo>(
                r#"
                SELECT id, parent_page, mimetype
                FROM image_chunk
                WHERE id = $1
                "#,
            )
            .bind(chunk_id)
            .fetch_optional(&pool)
            .await?
        } else {
            None
        };

        // Fetch page info if chunk has a parent_page
        let page = if let Some(ref c) = chunk {
            if let Some(parent_page) = c.parent_page {
                sqlx::query_as::<_, PageInfo>(
                    r#"
                    SELECT id, page_num, document_id, mimetype, page_metadata
                    FROM page
                    WHERE id = $1
                    "#,
                )
                .bind(parent_page)
                .fetch_optional(&pool)
                .await?
            } else {
                None
            }
        } else {
            None
        };

        let item = EvidenceItem {
            relation: relation.clone(),
            chunk,
            page,
        };

        groups_map
            .entry(relation.group_index)
            .or_default()
            .push(item);
    }

    // Convert to sorted Vec of EvidenceGroup
    let mut evidence_groups: Vec<EvidenceGroup> = groups_map
        .into_iter()
        .map(|(group_index, items)| EvidenceGroup { group_index, items })
        .collect();
    evidence_groups.sort_by_key(|g| g.group_index);

    Ok(QueryWithEvidence {
        query,
        evidence_groups,
    })
}

#[tauri::command]
pub async fn add_retrieval_relation(
    request: AddEvidenceRequest,
    state: State<'_, AppState>,
) -> Result<RetrievalRelation> {
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    // Find the next group_order for this group
    let max_order: (Option<i32>,) = sqlx::query_as(
        r#"
        SELECT MAX(group_order)
        FROM retrieval_relation
        WHERE query_id = $1 AND group_index = $2
        "#,
    )
    .bind(request.query_id)
    .bind(request.group_index)
    .fetch_one(&pool)
    .await?;

    let next_order = max_order.0.map(|o| o + 1).unwrap_or(0);

    let relation = sqlx::query_as::<_, RetrievalRelation>(
        r#"
        INSERT INTO retrieval_relation (query_id, group_index, group_order, image_chunk_id)
        VALUES ($1, $2, $3, $4)
        RETURNING query_id, group_index, group_order, chunk_id, image_chunk_id
        "#,
    )
    .bind(request.query_id)
    .bind(request.group_index)
    .bind(next_order)
    .bind(request.image_chunk_id)
    .fetch_one(&pool)
    .await?;

    Ok(relation)
}

#[tauri::command]
pub async fn remove_retrieval_relation(
    query_id: i64,
    group_index: i32,
    group_order: i32,
    state: State<'_, AppState>,
) -> Result<bool> {
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    sqlx::query(
        r#"
        DELETE FROM retrieval_relation
        WHERE query_id = $1 AND group_index = $2 AND group_order = $3
        "#,
    )
    .bind(query_id)
    .bind(group_index)
    .bind(group_order)
    .execute(&pool)
    .await?;

    // Reorder remaining items in the group
    sqlx::query(
        r#"
        UPDATE retrieval_relation
        SET group_order = group_order - 1
        WHERE query_id = $1 AND group_index = $2 AND group_order > $3
        "#,
    )
    .bind(query_id)
    .bind(group_index)
    .bind(group_order)
    .execute(&pool)
    .await?;

    Ok(true)
}

/// Remove all evidence from a specific group
#[tauri::command]
pub async fn remove_evidence_group(
    query_id: i64,
    group_index: i32,
    state: State<'_, AppState>,
) -> Result<bool> {
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    sqlx::query(
        r#"
        DELETE FROM retrieval_relation
        WHERE query_id = $1 AND group_index = $2
        "#,
    )
    .bind(query_id)
    .bind(group_index)
    .execute(&pool)
    .await?;

    // Reorder remaining groups
    sqlx::query(
        r#"
        UPDATE retrieval_relation
        SET group_index = group_index - 1
        WHERE query_id = $1 AND group_index > $2
        "#,
    )
    .bind(query_id)
    .bind(group_index)
    .execute(&pool)
    .await?;

    Ok(true)
}

/// Reorder evidence within a group
#[tauri::command]
pub async fn reorder_evidence(
    query_id: i64,
    group_index: i32,
    from_order: i32,
    to_order: i32,
    state: State<'_, AppState>,
) -> Result<bool> {
    let pool = state.get_pool().await.ok_or(AppError::NotConnected)?;

    if from_order == to_order {
        return Ok(true);
    }

    // Temporarily set the moving item to -1
    sqlx::query(
        r#"
        UPDATE retrieval_relation
        SET group_order = -1
        WHERE query_id = $1 AND group_index = $2 AND group_order = $3
        "#,
    )
    .bind(query_id)
    .bind(group_index)
    .bind(from_order)
    .execute(&pool)
    .await?;

    if from_order < to_order {
        // Moving down: shift items up
        sqlx::query(
            r#"
            UPDATE retrieval_relation
            SET group_order = group_order - 1
            WHERE query_id = $1 AND group_index = $2
            AND group_order > $3 AND group_order <= $4
            "#,
        )
        .bind(query_id)
        .bind(group_index)
        .bind(from_order)
        .bind(to_order)
        .execute(&pool)
        .await?;
    } else {
        // Moving up: shift items down
        sqlx::query(
            r#"
            UPDATE retrieval_relation
            SET group_order = group_order + 1
            WHERE query_id = $1 AND group_index = $2
            AND group_order >= $3 AND group_order < $4
            "#,
        )
        .bind(query_id)
        .bind(group_index)
        .bind(to_order)
        .bind(from_order)
        .execute(&pool)
        .await?;
    }

    // Set the final position
    sqlx::query(
        r#"
        UPDATE retrieval_relation
        SET group_order = $4
        WHERE query_id = $1 AND group_index = $2 AND group_order = -1
        "#,
    )
    .bind(query_id)
    .bind(group_index)
    .bind(to_order)
    .execute(&pool)
    .await?;

    Ok(true)
}
