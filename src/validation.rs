use regex::Regex;

// r"^[\w-\.]+@([\w-]+\.)+[\w-]{2,4}$ -> email
pub fn validate_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^([\w-]+[\.]*)+[\w-]+@([\w-]+\.)+[\w-]{2,4}$").unwrap();
    email_regex.is_match(email)
}

pub fn validate_password(password: &str) -> bool {
    // let password_regex = Regex::new(r"^[[\w-][@!#$\.&]+]{8,}$").unwrap();
    password.len() >= 8
}
