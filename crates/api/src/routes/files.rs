use crate::state::AppState;
use axum::{
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use storage::models::UploadOptions;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct FileUploadResponse {
    pub id: Uuid,
    pub key: String,
    pub bucket: String,
    pub filename: String,
    pub content_type: String,
    pub size: u64,
    pub url: Option<String>,
    pub sha256: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct FileInfoResponse {
    pub key: String,
    pub bucket: String,
    pub content_type: String,
    pub size: u64,
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize)]
pub struct PresignedUrlResponse {
    pub url: String,
    pub expires_in: u32,
    pub method: String,
}

#[derive(Debug, Deserialize)]
pub struct PresignedUrlQuery {
    pub expires_in: Option<u32>,
    pub filename: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UploadUrlQuery {
    pub expires_in: Option<u32>,
    pub filename: Option<String>,
    pub content_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListFilesQuery {
    pub prefix: Option<String>,
    pub limit: Option<usize>,
}

pub async fn upload_brochure(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<FileUploadResponse>, StatusCode> {
    let field = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .ok_or(StatusCode::BAD_REQUEST)?;

    let filename = field
        .file_name()
        .map(String::from)
        .unwrap_or_else(|| "file".into());
    let content_type = field.content_type().map(String::from).unwrap_or_else(|| {
        mime_guess::from_path(&filename)
            .first()
            .map(|m| m.to_string())
            .unwrap_or_else(|| "application/octet-stream".into())
    });

    let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;

    let storage = &state.storage_client;
    let bucket = "brochures";

    storage
        .create_bucket_if_not_exists(bucket)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "bucket creation failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let file_id = Uuid::new_v4();
    let ext = std::path::Path::new(&filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");
    let key = format!(
        "brochures/{}/{}.{}",
        chrono::Utc::now().format("%Y/%m"),
        file_id,
        ext
    );

    let opts = UploadOptions::new()
        .with_key(&key)
        .with_content_type(&content_type)
        .with_metadata("original_filename", &filename)
        .with_download_filename(&filename);

    let result = storage
        .upload_bytes(bucket, &data, opts)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "upload failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(FileUploadResponse {
        id: file_id,
        key: result.key,
        bucket: result.bucket,
        filename,
        content_type: result.content_type,
        size: result.size,
        url: result.url,
        sha256: result.sha256,
    }))
}

pub async fn upload_product_image(
    State(state): State<AppState>,
    Path(product_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<FileUploadResponse>, StatusCode> {
    let field = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .ok_or(StatusCode::BAD_REQUEST)?;

    let filename = field
        .file_name()
        .map(String::from)
        .unwrap_or_else(|| "image".into());
    let content_type = field
        .content_type()
        .map(String::from)
        .unwrap_or_else(|| "image/jpeg".into());

    if !content_type.starts_with("image/") {
        return Err(StatusCode::BAD_REQUEST);
    }

    let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;

    let storage = &state.storage_client;
    let bucket = "products";

    storage
        .create_bucket_if_not_exists(bucket)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let file_id = Uuid::new_v4();
    let ext = std::path::Path::new(&filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpg");
    let key = format!("products/{}/{}.{}", product_id, file_id, ext);

    let opts = UploadOptions::new()
        .with_key(&key)
        .with_content_type(&content_type)
        .public();

    let result = storage
        .upload_bytes(bucket, &data, opts)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "upload failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(FileUploadResponse {
        id: file_id,
        key: result.key,
        bucket: result.bucket,
        filename,
        content_type: result.content_type,
        size: result.size,
        url: result.url,
        sha256: result.sha256,
    }))
}

pub async fn get_download_url(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
    Query(query): Query<PresignedUrlQuery>,
) -> Result<Json<PresignedUrlResponse>, StatusCode> {
    let expires_in = query.expires_in.unwrap_or(3600);

    let url = state
        .storage_client
        .download_url(&bucket, &key, query.filename.as_deref(), expires_in)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "presigned url failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(PresignedUrlResponse {
        url: url.url,
        expires_in: url.expires_in,
        method: url.method,
    }))
}

pub async fn get_upload_url(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
    Query(query): Query<UploadUrlQuery>,
) -> Result<Json<PresignedUrlResponse>, StatusCode> {
    let expires_in = query.expires_in.unwrap_or(3600);
    let file_id = Uuid::new_v4();
    let ext = query
        .filename
        .as_ref()
        .and_then(|f| std::path::Path::new(f).extension())
        .and_then(|e| e.to_str())
        .unwrap_or("bin");

    let key = format!(
        "{}/{}/{}.{}",
        bucket,
        chrono::Utc::now().format("%Y/%m"),
        file_id,
        ext
    );

    let url = state
        .storage_client
        .upload_url(&bucket, &key, query.content_type.as_deref(), expires_in)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "presigned url failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(PresignedUrlResponse {
        url: url.url,
        expires_in: url.expires_in,
        method: url.method,
    }))
}

pub async fn get_file_info(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
) -> Result<Json<FileInfoResponse>, StatusCode> {
    let info = state
        .storage_client
        .object_info(&bucket, &key)
        .await
        .map_err(|e| match e {
            storage::StorageError::NotFound { .. } => StatusCode::NOT_FOUND,
            _ => {
                tracing::error!(error = %e, "get info failed");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    Ok(Json(FileInfoResponse {
        key: info.key,
        bucket: info.bucket,
        content_type: info.content_type,
        size: info.size,
        last_modified: info.last_modified,
    }))
}

pub async fn delete_file(
    State(state): State<AppState>,
    Path((bucket, key)): Path<(String, String)>,
) -> Result<StatusCode, StatusCode> {
    state
        .storage_client
        .delete(&bucket, &key)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "delete failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_files(
    State(state): State<AppState>,
    Path(bucket): Path<String>,
    Query(query): Query<ListFilesQuery>,
) -> Result<Json<Vec<FileInfoResponse>>, StatusCode> {
    let result = state
        .storage_client
        .list(&bucket, query.prefix.as_deref(), query.limit)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "list failed");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(
        result
            .objects
            .into_iter()
            .map(|obj| FileInfoResponse {
                key: obj.key,
                bucket: obj.bucket,
                content_type: obj.content_type,
                size: obj.size,
                last_modified: obj.last_modified,
            })
            .collect(),
    ))
}
