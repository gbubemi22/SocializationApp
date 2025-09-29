use regex::Regex;
use crate::utils::error::CustomError;

// pub fn validate_password(password: &str) -> Result<(), CustomError> {
//     // Regex pattern to check for length and character requirements
//     let re = Regex::new(r"^[a-zA-Z\d]{8,20}$").unwrap();

//     // Check if password length and character requirements are met
//     if !re.is_match(password) {
//         return Err(CustomError::BadRequestError("Password must be between 8 and 20 characters long and include at least one letter and one number.".into()));
//     }

//     // Additional checks for uppercase, lowercase, and digits
//     if !password.chars().any(|c| c.is_lowercase()) {
//         return Err(CustomError::BadRequestError("Password must contain at least one lowercase letter.".into()));
//     }
//     if !password.chars().any(|c| c.is_uppercase()) {
//         return Err(CustomError::BadRequestError("Password must contain at least one uppercase letter.".into()));
//     }
//     if !password.chars().any(|c| c.is_digit(10)) {
//         return Err(CustomError::BadRequestError("Password must contain at least one number.".into()));
//     }

//     Ok(())
// }

pub fn validate_password(password: &str) -> Result<(), CustomError> {
    // Check password length
    if password.len() < 8 || password.len() > 20 {
        return Err(CustomError::BadRequestError("Password must be between 8 and 20 characters long.".into()));
    }

    // Check for at least one lowercase letter, one uppercase letter, and one digit
    let has_lowercase = password.chars().any(|c| c.is_lowercase());
    let has_uppercase = password.chars().any(|c| c.is_uppercase());
    let has_digit = password.chars().any(|c| c.is_digit(10));

    if !has_lowercase || !has_uppercase || !has_digit {
        return Err(CustomError::BadRequestError("Password must include at least one uppercase letter, one lowercase letter, and one number.".into()));
    }

    Ok(())
}

// pub fn validate_password(password: &str) -> Result<(), String> {
//     let re = Regex::new(r"^(?=.*\d)(?=.*[a-z])(?=.*[A-Z]).{8,20}$").unwrap();
//     if !re.is_match(password) {
//         return Err(CustomError::BadRequestError("Password must contain a capital letter, number, special character & greater than 8 digits.".into()).to_string());
//     }
//     Ok(())
// }