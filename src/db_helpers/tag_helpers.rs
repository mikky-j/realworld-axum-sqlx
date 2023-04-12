use sqlx::SqlitePool;

use crate::errors::RequestError;

pub async fn get_tags_in_db(pool: &SqlitePool) -> Result<Vec<String>, RequestError> {
    let mut tx = pool.begin().await?;
    let result = sqlx::query!(
        r#"
        SELECT name from Tags
        "#
    )
    .fetch_all(&mut tx)
    .await?
    .into_iter()
    .map(|record| record.name)
    .collect();

    tx.commit().await?;
    Ok(result)
}
