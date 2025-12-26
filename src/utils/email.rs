use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use std::env;

/// SMTP Configuration for Zoho
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_email: String,
    pub from_name: String,
}

impl EmailConfig {
    /// Load email configuration from environment variables
    pub fn from_env() -> Result<Self, String> {
        Ok(Self {
            smtp_host: env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.zoho.com".to_string()),
            smtp_port: env::var("SMTP_PORT")
                .unwrap_or_else(|_| "465".to_string())
                .parse()
                .map_err(|_| "SMTP_PORT must be a valid number")?,
            smtp_username: env::var("SMTP_USERNAME").map_err(|_| "SMTP_USERNAME is required")?,
            smtp_password: env::var("SMTP_PASSWORD").map_err(|_| "SMTP_PASSWORD is required")?,
            from_email: env::var("SMTP_FROM_EMAIL").map_err(|_| "SMTP_FROM_EMAIL is required")?,
            from_name: env::var("SMTP_FROM_NAME")
                .unwrap_or_else(|_| "SocializationApp".to_string()),
        })
    }
}

/// Email service for sending emails via Zoho SMTP
pub struct EmailService {
    config: EmailConfig,
}

impl EmailService {
    /// Create a new EmailService instance
    pub fn new() -> Result<Self, String> {
        let config = EmailConfig::from_env()?;
        Ok(Self { config })
    }

    /// Create a new EmailService with custom config
    pub fn with_config(config: EmailConfig) -> Self {
        Self { config }
    }

    /// Build the SMTP transport
    fn build_transport(&self) -> Result<AsyncSmtpTransport<Tokio1Executor>, String> {
        let creds = Credentials::new(
            self.config.smtp_username.clone(),
            self.config.smtp_password.clone(),
        );

        // Zoho uses port 465 with implicit TLS (SMTPS)
        let transport = AsyncSmtpTransport::<Tokio1Executor>::relay(&self.config.smtp_host)
            .map_err(|e| format!("Failed to create SMTP transport: {}", e))?
            .credentials(creds)
            .port(self.config.smtp_port)
            .build();

        Ok(transport)
    }

    /// Send a plain text email
    pub async fn send_email(
        &self,
        to_email: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), String> {
        let from_address = format!("{} <{}>", self.config.from_name, self.config.from_email);

        let email = Message::builder()
            .from(
                from_address
                    .parse()
                    .map_err(|e| format!("Invalid from address: {}", e))?,
            )
            .to(to_email
                .parse()
                .map_err(|e| format!("Invalid to address: {}", e))?)
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body.to_string())
            .map_err(|e| format!("Failed to build email: {}", e))?;

        let transport = self.build_transport()?;

        transport
            .send(email)
            .await
            .map_err(|e| format!("Failed to send email: {}", e))?;

        Ok(())
    }

    /// Send an HTML email
    pub async fn send_html_email(
        &self,
        to_email: &str,
        subject: &str,
        html_body: &str,
    ) -> Result<(), String> {
        let from_address = format!("{} <{}>", self.config.from_name, self.config.from_email);

        let email = Message::builder()
            .from(
                from_address
                    .parse()
                    .map_err(|e| format!("Invalid from address: {}", e))?,
            )
            .to(to_email
                .parse()
                .map_err(|e| format!("Invalid to address: {}", e))?)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(html_body.to_string())
            .map_err(|e| format!("Failed to build email: {}", e))?;

        let transport = self.build_transport()?;

        transport
            .send(email)
            .await
            .map_err(|e| format!("Failed to send email: {}", e))?;

        Ok(())
    }

    /// Send a verification email with OTP
    pub async fn send_verification_email(
        &self,
        to_email: &str,
        otp_code: &str,
    ) -> Result<(), String> {
        let subject = "Verify Your Email - SocializationApp";
        let body = format!(
            "Welcome to SocializationApp!\n\n\
            Your verification code is: {}\n\n\
            This code will expire in 10 minutes.\n\n\
            If you didn't request this, please ignore this email.",
            otp_code
        );

        self.send_email(to_email, subject, &body).await
    }

    /// Send a password reset email
    pub async fn send_password_reset_email(
        &self,
        to_email: &str,
        reset_token: &str,
    ) -> Result<(), String> {
        let subject = "Password Reset - SocializationApp";
        let body = format!(
            "You requested a password reset.\n\n\
            Your reset token is: {}\n\n\
            This token will expire in 15 minutes.\n\n\
            If you didn't request this, please ignore this email.",
            reset_token
        );

        self.send_email(to_email, subject, &body).await
    }
}
