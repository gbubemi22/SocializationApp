use crate::database::RedisService;
use crate::middleware::auth::{create_token, create_token_with_session};
use crate::user::model::{Otp, User};
use crate::utils::email::EmailService;
use crate::utils::error::CustomError;
use crate::utils::helpers::{OTP_EXPIRATION_MINUTES, generate_otp_code};
use crate::utils::model::LoginRequests;
use crate::utils::{hashing, password_validation};
use chrono::{Duration, Utc};
use mongodb::bson::{doc, oid::ObjectId};
use mongodb::{Client, Collection};

pub struct UserService {
    collection: Collection<User>,
    otp_collection: Collection<Otp>,
}

impl UserService {
    pub fn new(client: &Client) -> Self {
        let db = client.database("rust_blogdb");
        let collection = db.collection::<User>("users");
        let otp_collection = db.collection::<Otp>("otps");

        UserService {
            collection,
            otp_collection,
        }
    }

    /// Create and store OTP for a user
    async fn create_otp(&self, user_id: ObjectId, email: &str) -> Result<String, CustomError> {
        let code = generate_otp_code();

        // Mark any existing unused OTPs as used
        let _ = self
            .otp_collection
            .update_many(
                doc! { "email": email, "is_used": false },
                doc! { "$set": { "is_used": true } },
            )
            .await;

        // Create new OTP
        let otp = Otp {
            id: None,
            user_id,
            email: email.to_string(),
            code: code.clone(),
            expires_at: Utc::now() + Duration::minutes(OTP_EXPIRATION_MINUTES),
            is_used: false,
            created_at: Utc::now(),
        };

        self.otp_collection
            .insert_one(otp)
            .await
            .map_err(|e| CustomError::InternalServerError(e.to_string()))?;

        Ok(code)
    }

    /// Send OTP email to user
    async fn send_otp_email(&self, email: &str, otp_code: &str) -> Result<(), CustomError> {
        let email_service = EmailService::new()
            .map_err(|e| CustomError::InternalServerError(format!("Email service error: {}", e)))?;

        email_service
            .send_verification_email(email, otp_code)
            .await
            .map_err(|e| {
                CustomError::InternalServerError(format!("Failed to send email: {}", e))
            })?;

        Ok(())
    }

    pub async fn create_user(
        &self,
        username: String,
        email: String,
        password: String,
        phone_number: String,
    ) -> Result<ObjectId, CustomError> {
        // Check if email already exists
        if self.email_exists(&email).await.map_err(|_| {
            CustomError::InternalServerError("Failed to check email existence".to_string())
        })? {
            return Err(CustomError::ConflictError(
                "Email already exists".to_string(),
            ));
        }

        // Check if username already exists
        if self.username_exists(&username).await.map_err(|_| {
            CustomError::InternalServerError("Failed to check username existence".to_string())
        })? {
            return Err(CustomError::ConflictError(
                "Username already exists".to_string(),
            ));
        }

        // Check if phone number already exists
        if self.phone_number(&phone_number).await.map_err(|_| {
            CustomError::InternalServerError("Failed to check phone number existence".to_string())
        })? {
            return Err(CustomError::ConflictError(
                "Phone number already exists".to_string(),
            ));
        }

        // Validate password
        let _ = password_validation::validate_password(&password);

        // Hash the password
        let hashed_password = hashing::hash_password(&password)
            .map_err(|e| CustomError::InternalServerError(e.to_string()))?;

        // Create new user (not verified yet)
        let new_user = User {
            id: None,
            username,
            email: email.clone(),
            phone_number,
            password: hashed_password,
            profile_picture: None,
            is_email_verified: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Insert the user
        let result = self
            .collection
            .insert_one(new_user)
            .await
            .map_err(|e| CustomError::InternalServerError(e.to_string()))?;

        // Get the inserted ID
        let user_id = result.inserted_id.as_object_id().ok_or_else(|| {
            CustomError::InternalServerError("Failed to get inserted ID".to_string())
        })?;

        // Generate and send OTP
        let otp_code = self.create_otp(user_id, &email).await?;
        self.send_otp_email(&email, &otp_code).await?;

        Ok(user_id)
    }

    /// Verify user's email with OTP
    pub async fn verify_email(&self, email: &str, otp_code: &str) -> Result<(), CustomError> {
        // Find the OTP
        let otp = self
            .otp_collection
            .find_one(doc! {
                "email": email,
                "code": otp_code,
                "is_used": false
            })
            .await
            .map_err(|e| CustomError::InternalServerError(e.to_string()))?
            .ok_or_else(|| CustomError::BadRequestError("Invalid OTP code".to_string()))?;

        // Check if OTP is expired
        if otp.expires_at < Utc::now() {
            return Err(CustomError::BadRequestError("OTP has expired".to_string()));
        }

        // Mark OTP as used
        self.otp_collection
            .update_one(doc! { "_id": otp.id }, doc! { "$set": { "is_used": true } })
            .await
            .map_err(|e| CustomError::InternalServerError(e.to_string()))?;

        // Update user's email verification status
        self.collection
            .update_one(
                doc! { "email": email },
                doc! {
                    "$set": {
                        "is_email_verified": true,
                        "updated_at": Utc::now().to_rfc3339()
                    }
                },
            )
            .await
            .map_err(|e| CustomError::InternalServerError(e.to_string()))?;

        Ok(())
    }

    /// Resend OTP to user's email
    pub async fn resend_otp(&self, email: &str) -> Result<(), CustomError> {
        // Find the user
        let user = self
            .collection
            .find_one(doc! { "email": email })
            .await
            .map_err(|e| CustomError::InternalServerError(e.to_string()))?
            .ok_or_else(|| CustomError::NotFoundError("User not found".to_string()))?;

        // Check if already verified
        if user.is_email_verified {
            return Err(CustomError::BadRequestError(
                "Email is already verified".to_string(),
            ));
        }

        let user_id = user
            .id
            .ok_or_else(|| CustomError::InternalServerError("User ID missing".to_string()))?;

        // Generate and send new OTP
        let otp_code = self.create_otp(user_id, email).await?;
        self.send_otp_email(email, &otp_code).await?;

        Ok(())
    }

    async fn email_exists(&self, email: &str) -> Result<bool, mongodb::error::Error> {
        let count = self
            .collection
            .count_documents(doc! { "email": email })
            .await?;
        Ok(count > 0)
    }

    async fn username_exists(&self, username: &str) -> Result<bool, mongodb::error::Error> {
        let count = self
            .collection
            .count_documents(doc! { "username": username })
            .await?;
        Ok(count > 0)
    }

    async fn phone_number(&self, phone_number: &str) -> Result<bool, mongodb::error::Error> {
        let count = self
            .collection
            .count_documents(doc! { "phone_number": phone_number})
            .await?;
        Ok(count > 0)
    }

    pub async fn authenticate_user(
        &self,
        username: &str,
        password: &str,
    ) -> Result<User, CustomError> {
        let user = self
            .collection
            .find_one(doc! { "username": username })
            .await
            .map_err(|_| CustomError::InternalServerError("Database error".to_string()))?
            .ok_or_else(|| CustomError::UnauthorizedError("Invalid credentials".to_string()))?;

        if !hashing::verify_password(password, &user.password)
            .map_err(|_| CustomError::InternalServerError("Invalid credentials".to_string()))?
        {
            return Err(CustomError::UnauthorizedError(
                "Invalid credentials".to_string(),
            ));
        }

        Ok(user)
    }

    pub async fn login_fn(
        &self,
        login_data: LoginRequests,
        redis_service: Option<&RedisService>,
    ) -> Result<String, CustomError> {
        // Authenticate user
        let user = self
            .authenticate_user(&login_data.username, &login_data.password)
            .await?;

        // Check if email is verified
        if !user.is_email_verified {
            return Err(CustomError::UnauthorizedError(
                "Please verify your email before logging in".to_string(),
            ));
        }

        // Generate JWT token
        let user_id = user
            .id
            .as_ref()
            .ok_or_else(|| CustomError::InternalServerError("User ID missing".to_string()))?;

        // Create token with Redis session if available
        let token = if let Some(redis) = redis_service {
            create_token_with_session(&user_id.to_hex(), redis)
                .await
                .map_err(|_| CustomError::BadRequestError("Token generation failed".to_string()))?
        } else {
            create_token(&user_id.to_hex())
                .await
                .map_err(|_| CustomError::BadRequestError("Token generation failed".to_string()))?
        };

        Ok(token)
    }
}
