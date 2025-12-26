use reqwest::multipart::{Form, Part};
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::env;

/// Cloudinary configuration loaded from environment variables
pub struct CloudinaryConfig {
    pub cloud_name: String,
    pub api_key: String,
    pub api_secret: String,
    pub upload_preset: Option<String>,
}

impl CloudinaryConfig {
    /// Load Cloudinary configuration from environment variables
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            cloud_name: env::var("CLOUDINARY_CLOUD_NAME")
                .map_err(|_| "CLOUDINARY_CLOUD_NAME is required")?,
            api_key: env::var("CLOUDINARY_API_KEY")
                .map_err(|_| "CLOUDINARY_API_KEY is required")?,
            api_secret: env::var("CLOUDINARY_API_SECRET")
                .map_err(|_| "CLOUDINARY_API_SECRET is required")?,
            upload_preset: env::var("CLOUDINARY_UPLOAD_PRESET").ok(),
        })
    }

    /// Get the upload URL for Cloudinary
    pub fn upload_url(&self, resource_type: &str) -> String {
        format!(
            "https://api.cloudinary.com/v1_1/{}/{}/upload",
            self.cloud_name, resource_type
        )
    }

    /// Generate a signature for authenticated uploads
    pub fn generate_signature(&self, params: &str, timestamp: i64) -> String {
        let to_sign = format!("{}&timestamp={}{}", params, timestamp, self.api_secret);
        let mut hasher = Sha1::new();
        hasher.update(to_sign.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

/// Response from Cloudinary upload API
#[derive(Debug, Deserialize, Serialize)]
pub struct CloudinaryUploadResponse {
    pub public_id: String,
    pub version: i64,
    pub signature: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub format: String,
    pub resource_type: String,
    pub created_at: String,
    pub bytes: u64,
    pub url: String,
    pub secure_url: String,
}

/// Cloudinary error response
#[derive(Debug, Deserialize)]
pub struct CloudinaryError {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct CloudinaryErrorResponse {
    pub error: CloudinaryError,
}

/// Upload service for Cloudinary
pub struct UploadService {
    config: CloudinaryConfig,
    client: reqwest::Client,
}

impl UploadService {
    /// Create a new UploadService instance
    pub fn new() -> Result<Self, String> {
        let config = CloudinaryConfig::from_env()?;
        let client = reqwest::Client::new();
        Ok(Self { config, client })
    }

    /// Create a new UploadService with custom config
    pub fn with_config(config: CloudinaryConfig) -> Self {
        let client = reqwest::Client::new();
        Self { config, client }
    }

    /// Upload an image to Cloudinary
    pub async fn upload_image(
        &self,
        file_data: Vec<u8>,
        file_name: &str,
        folder: Option<&str>,
    ) -> Result<CloudinaryUploadResponse, String> {
        self.upload_file(file_data, file_name, "image", folder)
            .await
    }

    /// Upload a video to Cloudinary
    pub async fn upload_video(
        &self,
        file_data: Vec<u8>,
        file_name: &str,
        folder: Option<&str>,
    ) -> Result<CloudinaryUploadResponse, String> {
        self.upload_file(file_data, file_name, "video", folder)
            .await
    }

    /// Upload a raw file to Cloudinary
    pub async fn upload_raw(
        &self,
        file_data: Vec<u8>,
        file_name: &str,
        folder: Option<&str>,
    ) -> Result<CloudinaryUploadResponse, String> {
        self.upload_file(file_data, file_name, "raw", folder).await
    }

    /// Generic file upload to Cloudinary
    async fn upload_file(
        &self,
        file_data: Vec<u8>,
        file_name: &str,
        resource_type: &str,
        folder: Option<&str>,
    ) -> Result<CloudinaryUploadResponse, String> {
        let timestamp = chrono::Utc::now().timestamp();
        let upload_url = self.config.upload_url(resource_type);

        // Build signature params
        let mut params = String::new();
        if let Some(f) = folder {
            params.push_str(&format!("folder={}", f));
        }
        if let Some(ref preset) = self.config.upload_preset {
            if !params.is_empty() {
                params.push('&');
            }
            params.push_str(&format!("upload_preset={}", preset));
        }

        let signature = self.config.generate_signature(&params, timestamp);

        // Build multipart form
        let file_part = Part::bytes(file_data)
            .file_name(file_name.to_string())
            .mime_str("application/octet-stream")
            .map_err(|e| format!("Failed to create file part: {}", e))?;

        let mut form = Form::new()
            .part("file", file_part)
            .text("api_key", self.config.api_key.clone())
            .text("timestamp", timestamp.to_string())
            .text("signature", signature);

        if let Some(f) = folder {
            form = form.text("folder", f.to_string());
        }

        if let Some(ref preset) = self.config.upload_preset {
            form = form.text("upload_preset", preset.clone());
        }

        // Send request
        let response = self
            .client
            .post(&upload_url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("Failed to send upload request: {}", e))?;

        if response.status().is_success() {
            response
                .json::<CloudinaryUploadResponse>()
                .await
                .map_err(|e| format!("Failed to parse upload response: {}", e))
        } else {
            let error_response = response
                .json::<CloudinaryErrorResponse>()
                .await
                .map_err(|e| format!("Failed to parse error response: {}", e))?;
            Err(format!(
                "Cloudinary upload failed: {}",
                error_response.error.message
            ))
        }
    }

    /// Upload image from base64 string
    pub async fn upload_image_base64(
        &self,
        base64_data: &str,
        folder: Option<&str>,
    ) -> Result<CloudinaryUploadResponse, String> {
        let timestamp = chrono::Utc::now().timestamp();
        let upload_url = self.config.upload_url("image");

        // Build signature params
        let mut params = String::new();
        if let Some(f) = folder {
            params.push_str(&format!("folder={}", f));
        }

        let signature = self.config.generate_signature(&params, timestamp);

        // Prepare base64 data URI
        let file_data = if base64_data.starts_with("data:") {
            base64_data.to_string()
        } else {
            format!("data:image/png;base64,{}", base64_data)
        };

        let mut form = Form::new()
            .text("file", file_data)
            .text("api_key", self.config.api_key.clone())
            .text("timestamp", timestamp.to_string())
            .text("signature", signature);

        if let Some(f) = folder {
            form = form.text("folder", f.to_string());
        }

        if let Some(ref preset) = self.config.upload_preset {
            form = form.text("upload_preset", preset.clone());
        }

        // Send request
        let response = self
            .client
            .post(&upload_url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("Failed to send upload request: {}", e))?;

        if response.status().is_success() {
            response
                .json::<CloudinaryUploadResponse>()
                .await
                .map_err(|e| format!("Failed to parse upload response: {}", e))
        } else {
            let error_response = response
                .json::<CloudinaryErrorResponse>()
                .await
                .map_err(|e| format!("Failed to parse error response: {}", e))?;
            Err(format!(
                "Cloudinary upload failed: {}",
                error_response.error.message
            ))
        }
    }

    /// Delete a resource from Cloudinary
    pub async fn delete_resource(
        &self,
        public_id: &str,
        resource_type: &str,
    ) -> Result<(), String> {
        let timestamp = chrono::Utc::now().timestamp();
        let destroy_url = format!(
            "https://api.cloudinary.com/v1_1/{}/{}/destroy",
            self.config.cloud_name, resource_type
        );

        let params = format!("public_id={}", public_id);
        let signature = self.config.generate_signature(&params, timestamp);

        let form = Form::new()
            .text("public_id", public_id.to_string())
            .text("api_key", self.config.api_key.clone())
            .text("timestamp", timestamp.to_string())
            .text("signature", signature);

        let response = self
            .client
            .post(&destroy_url)
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("Failed to send delete request: {}", e))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err("Failed to delete resource from Cloudinary".to_string())
        }
    }

    // ============================================
    // Single & Multiple File Upload with Validation
    // ============================================

    /// Upload a single file with validation
    pub async fn upload_single_file(
        &self,
        file: FileUpload,
        folder: Option<&str>,
        validator: &FileValidator,
    ) -> Result<CloudinaryUploadResponse, String> {
        // Validate the file
        validator.validate(&file)?;

        // Determine resource type based on file type
        let resource_type = validator.get_resource_type(&file.file_name);

        self.upload_file(file.data, &file.file_name, &resource_type, folder)
            .await
    }

    /// Upload multiple files with validation
    pub async fn upload_multiple_files(
        &self,
        files: Vec<FileUpload>,
        folder: Option<&str>,
        validator: &FileValidator,
    ) -> Result<Vec<UploadResult>, String> {
        if files.is_empty() {
            return Err("No files provided for upload".to_string());
        }

        // Validate max file count
        if let Some(max_count) = validator.max_file_count {
            if files.len() > max_count {
                return Err(format!(
                    "Too many files. Maximum allowed: {}, provided: {}",
                    max_count,
                    files.len()
                ));
            }
        }

        let mut results = Vec::new();

        for file in files {
            let result = match self
                .upload_single_file(file.clone(), folder, validator)
                .await
            {
                Ok(response) => UploadResult {
                    file_name: file.file_name,
                    success: true,
                    response: Some(response),
                    error: None,
                },
                Err(e) => UploadResult {
                    file_name: file.file_name,
                    success: false,
                    response: None,
                    error: Some(e),
                },
            };
            results.push(result);
        }

        Ok(results)
    }
}

// ============================================
// File Upload & Validation Structs
// ============================================

/// Represents a file to be uploaded
#[derive(Debug, Clone)]
pub struct FileUpload {
    pub file_name: String,
    pub data: Vec<u8>,
    pub content_type: Option<String>,
}

impl FileUpload {
    /// Create a new FileUpload
    pub fn new(file_name: String, data: Vec<u8>, content_type: Option<String>) -> Self {
        Self {
            file_name,
            data,
            content_type,
        }
    }

    /// Get file size in bytes
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Get file extension
    pub fn extension(&self) -> Option<String> {
        self.file_name
            .rsplit('.')
            .next()
            .map(|ext| ext.to_lowercase())
    }
}

/// Result of a single file upload in batch operations
#[derive(Debug, Serialize)]
pub struct UploadResult {
    pub file_name: String,
    pub success: bool,
    pub response: Option<CloudinaryUploadResponse>,
    pub error: Option<String>,
}

/// File validation configuration
#[derive(Debug, Clone)]
pub struct FileValidator {
    /// Allowed file extensions (e.g., ["jpg", "png", "gif"])
    pub allowed_extensions: Vec<String>,
    /// Maximum file size in bytes
    pub max_file_size: usize,
    /// Minimum file size in bytes (optional)
    pub min_file_size: Option<usize>,
    /// Maximum number of files for batch uploads
    pub max_file_count: Option<usize>,
}

impl FileValidator {
    /// Create a new FileValidator with default settings
    /// Default: images only, max 5MB
    pub fn new() -> Self {
        Self {
            allowed_extensions: vec![
                "jpg".to_string(),
                "jpeg".to_string(),
                "png".to_string(),
                "gif".to_string(),
                "webp".to_string(),
            ],
            max_file_size: 5 * 1024 * 1024, // 5MB
            min_file_size: None,
            max_file_count: Some(10),
        }
    }

    /// Create validator for images only
    pub fn images() -> Self {
        Self {
            allowed_extensions: vec![
                "jpg".to_string(),
                "jpeg".to_string(),
                "png".to_string(),
                "gif".to_string(),
                "webp".to_string(),
                "svg".to_string(),
                "bmp".to_string(),
            ],
            max_file_size: 10 * 1024 * 1024, // 10MB
            min_file_size: Some(1024),       // 1KB minimum
            max_file_count: Some(10),
        }
    }

    /// Create validator for videos only
    pub fn videos() -> Self {
        Self {
            allowed_extensions: vec![
                "mp4".to_string(),
                "mov".to_string(),
                "avi".to_string(),
                "mkv".to_string(),
                "webm".to_string(),
            ],
            max_file_size: 100 * 1024 * 1024, // 100MB
            min_file_size: Some(1024),
            max_file_count: Some(5),
        }
    }

    /// Create validator for documents
    pub fn documents() -> Self {
        Self {
            allowed_extensions: vec![
                "pdf".to_string(),
                "doc".to_string(),
                "docx".to_string(),
                "txt".to_string(),
                "xls".to_string(),
                "xlsx".to_string(),
            ],
            max_file_size: 25 * 1024 * 1024, // 25MB
            min_file_size: None,
            max_file_count: Some(10),
        }
    }

    /// Builder: Set allowed extensions
    pub fn with_extensions(mut self, extensions: Vec<&str>) -> Self {
        self.allowed_extensions = extensions.iter().map(|s| s.to_lowercase()).collect();
        self
    }

    /// Builder: Set max file size in bytes
    pub fn with_max_size(mut self, size_bytes: usize) -> Self {
        self.max_file_size = size_bytes;
        self
    }

    /// Builder: Set max file size in MB
    pub fn with_max_size_mb(mut self, size_mb: usize) -> Self {
        self.max_file_size = size_mb * 1024 * 1024;
        self
    }

    /// Builder: Set min file size in bytes
    pub fn with_min_size(mut self, size_bytes: usize) -> Self {
        self.min_file_size = Some(size_bytes);
        self
    }

    /// Builder: Set max file count for batch uploads
    pub fn with_max_count(mut self, count: usize) -> Self {
        self.max_file_count = Some(count);
        self
    }

    /// Validate a file
    pub fn validate(&self, file: &FileUpload) -> Result<(), String> {
        // Check file extension
        let extension = file.extension().ok_or("File has no extension")?;

        if !self.allowed_extensions.contains(&extension) {
            return Err(format!(
                "Invalid file type '{}'. Allowed types: {}",
                extension,
                self.allowed_extensions.join(", ")
            ));
        }

        // Check max file size
        if file.size() > self.max_file_size {
            return Err(format!(
                "File too large. Maximum size: {} bytes, file size: {} bytes",
                self.max_file_size,
                file.size()
            ));
        }

        // Check min file size
        if let Some(min_size) = self.min_file_size {
            if file.size() < min_size {
                return Err(format!(
                    "File too small. Minimum size: {} bytes, file size: {} bytes",
                    min_size,
                    file.size()
                ));
            }
        }

        // Check if file is empty
        if file.data.is_empty() {
            return Err("File is empty".to_string());
        }

        Ok(())
    }

    /// Get Cloudinary resource type based on file extension
    pub fn get_resource_type(&self, file_name: &str) -> String {
        let extension = file_name
            .rsplit('.')
            .next()
            .map(|ext| ext.to_lowercase())
            .unwrap_or_default();

        match extension.as_str() {
            "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" | "bmp" | "ico" => "image".to_string(),
            "mp4" | "mov" | "avi" | "mkv" | "webm" | "flv" | "wmv" => "video".to_string(),
            _ => "raw".to_string(),
        }
    }

    /// Format file size for display
    pub fn format_size(bytes: usize) -> String {
        const KB: usize = 1024;
        const MB: usize = KB * 1024;
        const GB: usize = MB * 1024;

        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} bytes", bytes)
        }
    }
}

impl Default for FileValidator {
    fn default() -> Self {
        Self::new()
    }
}
