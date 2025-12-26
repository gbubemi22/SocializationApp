use actix_multipart::Multipart;
use actix_web::{HttpResponse, Responder};
use futures_util::StreamExt;
use serde::Serialize;
use serde_json::json;

use crate::utils::uploads::{FileUpload, FileValidator, UploadService};

/// Response for single file upload
#[derive(Debug, Serialize)]
pub struct SingleUploadResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<UploadData>,
}

/// Upload data returned after successful upload
#[derive(Debug, Serialize)]
pub struct UploadData {
    pub public_id: String,
    pub url: String,
    pub secure_url: String,
    pub format: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub bytes: u64,
}

/// Response for multiple file upload
#[derive(Debug, Serialize)]
pub struct MultipleUploadResponse {
    pub success: bool,
    pub message: String,
    pub total_files: usize,
    pub successful_uploads: usize,
    pub failed_uploads: usize,
    pub data: Vec<MultipleUploadData>,
}

#[derive(Debug, Serialize)]
pub struct MultipleUploadData {
    pub file_name: String,
    pub success: bool,
    pub data: Option<UploadData>,
    pub error: Option<String>,
}

/// Helper function to extract files from multipart form
async fn extract_files_from_multipart(mut payload: Multipart) -> Result<Vec<FileUpload>, String> {
    let mut files = Vec::new();

    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| format!("Error reading multipart field: {}", e))?;

        // Get content disposition - skip if not present
        let content_disposition = match field.content_disposition() {
            Some(cd) => cd,
            None => continue,
        };

        let field_name = content_disposition.get_name().unwrap_or("");

        // Only process file fields
        if field_name == "file" || field_name == "files" {
            let file_name = content_disposition
                .get_filename()
                .map(|f| f.to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let content_type = field.content_type().map(|ct| ct.to_string());

            let mut data = Vec::new();
            while let Some(chunk) = field.next().await {
                let chunk = chunk.map_err(|e| format!("Error reading file chunk: {}", e))?;
                data.extend_from_slice(&chunk);
            }

            if !data.is_empty() {
                files.push(FileUpload::new(file_name, data, content_type));
            }
        }
    }

    Ok(files)
}

/// Upload a single file
/// POST /upload/single
pub async fn upload_single(payload: Multipart) -> impl Responder {
    // Extract files from multipart
    let files = match extract_files_from_multipart(payload).await {
        Ok(f) => f,
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "success": false,
                "message": e,
                "data": null
            }));
        }
    };

    // Check if file was provided
    if files.is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "message": "No file provided",
            "data": null
        }));
    }

    // Get the first file
    let file = files.into_iter().next().unwrap();

    // Create upload service
    let upload_service = match UploadService::new() {
        Ok(s) => s,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "message": format!("Upload service error: {}", e),
                "data": null
            }));
        }
    };

    // Create validator for images
    let validator = FileValidator::images();

    // Upload the file
    match upload_service
        .upload_single_file(file, Some("uploads"), &validator)
        .await
    {
        Ok(response) => HttpResponse::Ok().json(SingleUploadResponse {
            success: true,
            message: "File uploaded successfully".to_string(),
            data: Some(UploadData {
                public_id: response.public_id,
                url: response.url,
                secure_url: response.secure_url,
                format: response.format,
                width: response.width,
                height: response.height,
                bytes: response.bytes,
            }),
        }),
        Err(e) => HttpResponse::BadRequest().json(json!({
            "success": false,
            "message": e,
            "data": null
        })),
    }
}

/// Upload multiple files
/// POST /upload/multiple
pub async fn upload_multiple(payload: Multipart) -> impl Responder {
    // Extract files from multipart
    let files = match extract_files_from_multipart(payload).await {
        Ok(f) => f,
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "success": false,
                "message": e,
                "total_files": 0,
                "successful_uploads": 0,
                "failed_uploads": 0,
                "data": []
            }));
        }
    };

    // Check if files were provided
    if files.is_empty() {
        return HttpResponse::BadRequest().json(json!({
            "success": false,
            "message": "No files provided",
            "total_files": 0,
            "successful_uploads": 0,
            "failed_uploads": 0,
            "data": []
        }));
    }

    let total_files = files.len();

    // Create upload service
    let upload_service = match UploadService::new() {
        Ok(s) => s,
        Err(e) => {
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "message": format!("Upload service error: {}", e),
                "total_files": total_files,
                "successful_uploads": 0,
                "failed_uploads": total_files,
                "data": []
            }));
        }
    };

    // Create validator for images
    let validator = FileValidator::images();

    // Upload all files
    match upload_service
        .upload_multiple_files(files, Some("uploads"), &validator)
        .await
    {
        Ok(results) => {
            let successful_uploads = results.iter().filter(|r| r.success).count();
            let failed_uploads = results.iter().filter(|r| !r.success).count();

            let data: Vec<MultipleUploadData> = results
                .into_iter()
                .map(|r| MultipleUploadData {
                    file_name: r.file_name,
                    success: r.success,
                    data: r.response.map(|resp| UploadData {
                        public_id: resp.public_id,
                        url: resp.url,
                        secure_url: resp.secure_url,
                        format: resp.format,
                        width: resp.width,
                        height: resp.height,
                        bytes: resp.bytes,
                    }),
                    error: r.error,
                })
                .collect();

            HttpResponse::Ok().json(MultipleUploadResponse {
                success: failed_uploads == 0,
                message: if failed_uploads == 0 {
                    "All files uploaded successfully".to_string()
                } else {
                    format!(
                        "{} of {} files uploaded successfully",
                        successful_uploads, total_files
                    )
                },
                total_files,
                successful_uploads,
                failed_uploads,
                data,
            })
        }
        Err(e) => HttpResponse::BadRequest().json(json!({
            "success": false,
            "message": e,
            "total_files": total_files,
            "successful_uploads": 0,
            "failed_uploads": total_files,
            "data": []
        })),
    }
}
