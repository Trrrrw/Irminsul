use std::sync::OnceLock;

use tokio::sync::Mutex;

use crate::{
    admin::services::embedding,
    content::{db, dto::content::EmbeddingJobView, model::EmbeddingJobStatus},
};

/// 用互斥锁保证当前进程只运行一个向量任务消费者，避免并发任务互相踩状态。
static JOB_RUNNER: OnceLock<Mutex<()>> = OnceLock::new();

/// 为某条语言文本创建一个向量任务，并异步触发处理。
pub async fn enqueue_for_text(
    content_item_id: i64,
    content_item_text_id: i64,
    trigger_reason: &str,
    requested_by_user_id: Option<i64>,
    requested_by_label: Option<String>,
) -> Result<i64, String> {
    let model = embedding::get_settings()
        .await
        .map_err(|error| error.to_string())?
        .current_model;
    let conn = db::database()
        .connect()
        .map_err(|error| format!("打开内容库连接失败: {error}"))?;
    let now = crate::admin::middlewares::auth::unix_timestamp();
    conn.execute(
        "INSERT INTO embedding_jobs
         (content_item_id, content_item_text_id, trigger_reason, status, model, attempt_count, requested_by_user_id, requested_by_label, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7, ?8, ?8)",
        turso::params![
            content_item_id,
            content_item_text_id,
            trigger_reason,
            EmbeddingJobStatus::Pending.as_str(),
            model,
            requested_by_user_id,
            requested_by_label,
            now,
        ],
    )
    .await
    .map_err(|error| format!("写入向量任务失败: {error}"))?;
    let job_id = last_insert_rowid(&conn).await?;
    spawn_processor();
    Ok(job_id)
}

/// 为某个内容实例的全部语言文本创建重建任务。
pub async fn enqueue_for_item(
    content_item_id: i64,
    requested_by_user_id: Option<i64>,
    requested_by_label: Option<String>,
) -> Result<usize, String> {
    let conn = db::database()
        .connect()
        .map_err(|error| format!("打开内容库连接失败: {error}"))?;
    let mut rows = conn
        .query(
            "SELECT id FROM content_item_texts WHERE content_item_id = ?1 ORDER BY id ASC",
            turso::params![content_item_id],
        )
        .await
        .map_err(|error| format!("查询内容文本失败: {error}"))?;
    let mut count = 0;
    while let Some(row) = rows
        .next()
        .await
        .map_err(|error| format!("读取内容文本失败: {error}"))?
    {
        let text_id: i64 = row.get(0).map_err(|error| error.to_string())?;
        enqueue_for_text(
            content_item_id,
            text_id,
            "manual_rebuild",
            requested_by_user_id,
            requested_by_label.clone(),
        )
        .await?;
        count += 1;
    }
    Ok(count)
}

/// 在模型修改后，将全部文本重新入队。
pub async fn enqueue_full_rebuild(
    requested_by_user_id: Option<i64>,
    requested_by_label: Option<String>,
) -> Result<usize, String> {
    let conn = db::database()
        .connect()
        .map_err(|error| format!("打开内容库连接失败: {error}"))?;
    let mut rows = conn
        .query(
            "SELECT id, content_item_id FROM content_item_texts ORDER BY id ASC",
            (),
        )
        .await
        .map_err(|error| format!("查询全部内容文本失败: {error}"))?;
    let mut count = 0;
    while let Some(row) = rows
        .next()
        .await
        .map_err(|error| format!("读取全部内容文本失败: {error}"))?
    {
        let text_id: i64 = row.get(0).map_err(|error| error.to_string())?;
        let item_id: i64 = row.get(1).map_err(|error| error.to_string())?;
        enqueue_for_text(
            item_id,
            text_id,
            "model_changed_rebuild",
            requested_by_user_id,
            requested_by_label.clone(),
        )
        .await?;
        count += 1;
    }
    Ok(count)
}

/// 列出向量任务。
pub async fn list_jobs(content_item_id: Option<i64>) -> Result<Vec<EmbeddingJobView>, String> {
    let conn = db::database()
        .connect()
        .map_err(|error| format!("打开内容库连接失败: {error}"))?;
    let mut rows = if let Some(content_item_id) = content_item_id {
        conn.query(
            "SELECT id, content_item_id, content_item_text_id, trigger_reason, status, model, error_message,
                    attempt_count, requested_by_user_id, requested_by_label, created_at, updated_at, started_at, completed_at
             FROM embedding_jobs WHERE content_item_id = ?1 ORDER BY id DESC",
            turso::params![content_item_id],
        )
        .await
        .map_err(|error| format!("查询向量任务失败: {error}"))?
    } else {
        conn.query(
            "SELECT id, content_item_id, content_item_text_id, trigger_reason, status, model, error_message,
                    attempt_count, requested_by_user_id, requested_by_label, created_at, updated_at, started_at, completed_at
             FROM embedding_jobs ORDER BY id DESC",
            (),
        )
        .await
        .map_err(|error| format!("查询向量任务失败: {error}"))?
    };

    let mut jobs = Vec::new();
    while let Some(row) = rows
        .next()
        .await
        .map_err(|error| format!("读取向量任务失败: {error}"))?
    {
        jobs.push(map_job_row(&row)?);
    }
    Ok(jobs)
}

/// 手动重试指定任务，本质上是重新按当前模型再入队一条新任务。
pub async fn retry_job(
    job_id: i64,
    requested_by_user_id: Option<i64>,
    requested_by_label: Option<String>,
) -> Result<i64, String> {
    let conn = db::database()
        .connect()
        .map_err(|error| format!("打开内容库连接失败: {error}"))?;
    let mut rows = conn
        .query(
            "SELECT content_item_id, content_item_text_id FROM embedding_jobs WHERE id = ?1",
            turso::params![job_id],
        )
        .await
        .map_err(|error| format!("查询向量任务失败: {error}"))?;
    let row = rows
        .next()
        .await
        .map_err(|error| format!("读取向量任务失败: {error}"))?
        .ok_or_else(|| "embedding_job_not_found".to_string())?;
    let content_item_id: i64 = row.get(0).map_err(|error| error.to_string())?;
    let content_item_text_id: i64 = row.get(1).map_err(|error| error.to_string())?;
    enqueue_for_text(
        content_item_id,
        content_item_text_id,
        "manual_retry",
        requested_by_user_id,
        requested_by_label,
    )
    .await
}

fn spawn_processor() {
    tokio::spawn(async {
        let lock = JOB_RUNNER.get_or_init(|| Mutex::new(()));
        let _guard = lock.lock().await;
        loop {
            let next_job = match next_pending_job().await {
                Ok(job) => job,
                Err(error) => {
                    tracing::error!("failed to load next embedding job: {error}");
                    return;
                }
            };
            let Some(job) = next_job else {
                return;
            };
            if let Err(error) = process_job(job).await {
                tracing::error!("failed to process embedding job: {error}");
            }
        }
    });
}

async fn next_pending_job() -> Result<Option<EmbeddingJobView>, String> {
    let conn = db::database()
        .connect()
        .map_err(|error| format!("打开内容库连接失败: {error}"))?;
    let mut rows = conn
        .query(
            "SELECT id, content_item_id, content_item_text_id, trigger_reason, status, model, error_message,
                    attempt_count, requested_by_user_id, requested_by_label, created_at, updated_at, started_at, completed_at
             FROM embedding_jobs WHERE status = ?1 ORDER BY id ASC LIMIT 1",
            turso::params![EmbeddingJobStatus::Pending.as_str()],
        )
        .await
        .map_err(|error| format!("查询待处理任务失败: {error}"))?;
    let Some(row) = rows
        .next()
        .await
        .map_err(|error| format!("读取待处理任务失败: {error}"))?
    else {
        return Ok(None);
    };
    Ok(Some(map_job_row(&row)?))
}

async fn process_job(job: EmbeddingJobView) -> Result<(), String> {
    let conn = db::database()
        .connect()
        .map_err(|error| format!("打开内容库连接失败: {error}"))?;
    let now = crate::admin::middlewares::auth::unix_timestamp();
    conn.execute(
        "UPDATE embedding_jobs SET status = ?1, attempt_count = attempt_count + 1, started_at = ?2, updated_at = ?2 WHERE id = ?3",
        turso::params![EmbeddingJobStatus::Processing.as_str(), now, job.id],
    )
    .await
    .map_err(|error| format!("更新任务状态失败: {error}"))?;

    let result = process_job_inner(&conn, &job).await;
    let finish_at = crate::admin::middlewares::auth::unix_timestamp();
    match result {
        Ok(()) => {
            conn.execute(
                "UPDATE embedding_jobs SET status = ?1, error_message = NULL, completed_at = ?2, updated_at = ?2 WHERE id = ?3",
                turso::params![EmbeddingJobStatus::Completed.as_str(), finish_at, job.id],
            )
            .await
            .map_err(|error| format!("更新任务完成状态失败: {error}"))?;
            Ok(())
        }
        Err(error_message) => {
            conn.execute(
                "UPDATE embedding_jobs SET status = ?1, error_message = ?2, updated_at = ?3 WHERE id = ?4",
                turso::params![
                    EmbeddingJobStatus::Failed.as_str(),
                    error_message.clone(),
                    finish_at,
                    job.id,
                ],
            )
            .await
            .map_err(|error| format!("更新任务失败状态失败: {error}"))?;
            Err(error_message)
        }
    }
}

async fn process_job_inner(conn: &turso::Connection, job: &EmbeddingJobView) -> Result<(), String> {
    let mut rows = conn
        .query(
            "SELECT ci.id, cit.id, g.code, ct.code, cit.locale, cit.name, cit.subtitle, cit.author, cit.summary, cit.body
             FROM content_item_texts cit
             JOIN content_items ci ON ci.id = cit.content_item_id
             JOIN content_types ct ON ct.id = ci.content_type_id
             JOIN games g ON g.id = ci.game_id
             WHERE cit.id = ?1",
            turso::params![job.content_item_text_id],
        )
        .await
        .map_err(|error| format!("查询任务内容失败: {error}"))?;
    let row = rows
        .next()
        .await
        .map_err(|error| format!("读取任务内容失败: {error}"))?
        .ok_or_else(|| "对应的内容文本不存在".to_string())?;

    let content_item_id: i64 = row.get(0).map_err(|error| error.to_string())?;
    let content_item_text_id: i64 = row.get(1).map_err(|error| error.to_string())?;
    let game_code: String = row.get(2).map_err(|error| error.to_string())?;
    let content_type_code: String = row.get(3).map_err(|error| error.to_string())?;
    let locale: String = row.get(4).map_err(|error| error.to_string())?;
    let name: String = row.get(5).map_err(|error| error.to_string())?;
    let subtitle: Option<String> = row.get(6).map_err(|error| error.to_string())?;
    let author: Option<String> = row.get(7).map_err(|error| error.to_string())?;
    let summary: Option<String> = row.get(8).map_err(|error| error.to_string())?;
    let body: Option<String> = row.get(9).map_err(|error| error.to_string())?;

    let mut runtime = embedding::get_runtime_embedding_config().await?;
    runtime.model = job.model.clone();
    let (title, canonical_text, chunks) = crate::vector::service::build_document(
        &name,
        subtitle.as_deref(),
        author.as_deref(),
        summary.as_deref(),
        body.as_deref(),
    );

    let now = crate::admin::middlewares::auth::unix_timestamp();
    let document_id = if let Some(document_id) =
        find_document_id(conn, content_item_text_id).await?
    {
        conn.execute(
            "UPDATE knowledge_documents
             SET content_item_id = ?1, game_code = ?2, content_type_code = ?3, locale = ?4, title = ?5, canonical_text = ?6, model = ?7, updated_at = ?8
             WHERE id = ?9",
            turso::params![
                content_item_id,
                game_code.clone(),
                content_type_code.clone(),
                locale.clone(),
                title.clone(),
                canonical_text.clone(),
                job.model.clone(),
                now,
                document_id,
            ],
        )
        .await
        .map_err(|error| format!("更新知识文档失败: {error}"))?;
        conn.execute(
            "DELETE FROM knowledge_chunks WHERE document_id = ?1",
            turso::params![document_id],
        )
        .await
        .map_err(|error| format!("删除旧分片失败: {error}"))?;
        document_id
    } else {
        conn.execute(
            "INSERT INTO knowledge_documents
             (content_item_id, content_item_text_id, game_code, content_type_code, locale, title, canonical_text, model, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            turso::params![
                content_item_id,
                content_item_text_id,
                game_code.clone(),
                content_type_code.clone(),
                locale.clone(),
                title,
                canonical_text,
                job.model.clone(),
                now,
            ],
        )
        .await
        .map_err(|error| format!("创建知识文档失败: {error}"))?;
        last_insert_rowid(conn).await?
    };

    for (chunk_index, chunk) in chunks.iter().enumerate() {
        let embedding_json = crate::vector::service::request_embedding(&runtime, chunk).await?;
        conn.execute(
            "INSERT INTO knowledge_chunks
             (document_id, content_item_id, content_item_text_id, game_code, content_type_code, locale, chunk_index, text, embedding, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, vector32(?9), ?10, ?10)",
            turso::params![
                document_id,
                content_item_id,
                content_item_text_id,
                game_code.clone(),
                content_type_code.clone(),
                locale.clone(),
                chunk_index as i64,
                chunk.clone(),
                embedding_json,
                now,
            ],
        )
        .await
        .map_err(|error| format!("写入知识分片失败: {error}"))?;
    }

    Ok(())
}

async fn find_document_id(
    conn: &turso::Connection,
    content_item_text_id: i64,
) -> Result<Option<i64>, String> {
    let mut rows = conn
        .query(
            "SELECT id FROM knowledge_documents WHERE content_item_text_id = ?1",
            turso::params![content_item_text_id],
        )
        .await
        .map_err(|error| format!("查询知识文档失败: {error}"))?;
    let Some(row) = rows
        .next()
        .await
        .map_err(|error| format!("读取知识文档失败: {error}"))?
    else {
        return Ok(None);
    };
    row.get(0)
        .map(Some)
        .map_err(|error| format!("解析知识文档 ID 失败: {error}"))
}

async fn last_insert_rowid(conn: &turso::Connection) -> Result<i64, String> {
    Ok(conn.last_insert_rowid())
}

fn map_job_row(row: &turso::Row) -> Result<EmbeddingJobView, String> {
    let status: String = row.get(4).map_err(|error| error.to_string())?;
    Ok(EmbeddingJobView {
        id: row.get(0).map_err(|error| error.to_string())?,
        content_item_id: row.get(1).map_err(|error| error.to_string())?,
        content_item_text_id: row.get(2).map_err(|error| error.to_string())?,
        trigger_reason: row.get(3).map_err(|error| error.to_string())?,
        status: status
            .parse::<EmbeddingJobStatus>()
            .map_err(|error| error.to_string())?,
        model: row.get(5).map_err(|error| error.to_string())?,
        error_message: row.get(6).map_err(|error| error.to_string())?,
        attempt_count: row.get(7).map_err(|error| error.to_string())?,
        requested_by_user_id: row.get(8).map_err(|error| error.to_string())?,
        requested_by_label: row.get(9).map_err(|error| error.to_string())?,
        created_at: row.get(10).map_err(|error| error.to_string())?,
        updated_at: row.get(11).map_err(|error| error.to_string())?,
        started_at: row.get(12).map_err(|error| error.to_string())?,
        completed_at: row.get(13).map_err(|error| error.to_string())?,
    })
}
