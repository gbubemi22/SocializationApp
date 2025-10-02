use crate::middleware::auth::create_token;
use crate::user::model::User;
use crate::utils::error::CustomError;
use crate::utils::model::LoginRequests;
use crate::utils::{hashing, password_validation};
use chrono::Utc;
use mongodb::bson::{doc, oid::ObjectId};
use mongodb::{Client, Collection};


pub struct UserService {
    collection: Collection<User>,
}

impl UserService {
    pub fn new(client: &Client) -> Self {
        let collection = client.database("rust_blogdb").collection::<User>("users"); // specify model type

        UserService { collection }
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

        // Check if phone number already exites
        if self.phone_number(&phone_number).await.map_err(|_| {
            CustomError::InternalServerError("Failed to check phone number existence".to_string())
        })? {
            return Err(CustomError::ConflictError(
                "Phone number already exists".to_string(),
            ));
        }
        eprintln!("❌ Checked phone_number existence");
        // Validate password
        let _ = password_validation::validate_password(&password);

        // Hash the password
        let hashed_password = hashing::hash_password(&password)
            .map_err(|e| CustomError::InternalServerError(e.to_string()))?;

        // Create new user
        let new_user = User {
            id: None,
            username,
            email,
            phone_number,
            password: hashed_password,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Insert the user
        let result = self
            .collection
            .insert_one(new_user)
            .await
            .map_err(|e| CustomError::InternalServerError(e.to_string()))?;

        // Return the inserted ID
        result.inserted_id.as_object_id().ok_or_else(|| {
            eprintln!("❌ MongoDB insert failed: inserted_id is not an ObjectId");
            CustomError::InternalServerError("Failed to get inserted ID".to_string())
        })
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

    pub async fn login_fn(&self, login_data: LoginRequests) -> Result<String, CustomError> {
        // Authenticate user
        let user = self
            .authenticate_user(&login_data.username, &login_data.password)
            .await?;

        // Generate JWT token

        let user_id = user
            .id
            .as_ref()
            .ok_or_else(|| CustomError::InternalServerError("User ID missing".to_string()))?;

        let token = create_token(&user_id.to_hex())
            .await
            .map_err(|_| CustomError::BadRequestError("Token generation failed".to_string()))?;

        Ok(token)
    }
}
