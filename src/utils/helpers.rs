use rand::Rng;

/// Generate a 6-digit OTP code
pub fn generate_otp_code() -> String {
    let mut rng = rand::rng();
    let code: u32 = rng.random_range(100000..999999);
    code.to_string()
}

/// OTP expiration time in minutes
pub const OTP_EXPIRATION_MINUTES: i64 = 10;
