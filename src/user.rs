use crate::{
    response::StatusResponse,
    token::generate_jwt_token,
    validation::{validate_email, validate_password},
    DBPool, DBPooledConnection,
};

use super::schema::users;
use actix_web::{
    post,
    web::{self, Data, Json},
    HttpResponse,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use diesel::{
    result::Error::NotFound, ExpressionMethods, Insertable, QueryDsl, Queryable, RunQueryDsl,
};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    email: String,
    name: String,
    dateofbirth: Option<String>,
    contact: Option<String>,
    password: String,
}

impl User {
    pub fn new_user(&self) -> Self {
        Self {
            email: self.email.to_string(),
            name: self.name.to_string(),
            dateofbirth: self.dateofbirth.clone(),
            contact: self.contact.clone(),
            password: self.password.to_string(),
        }
    }

    pub fn to_user_db(&self) -> UserDB {
        let hashed_password = hash(&self.password, DEFAULT_COST).unwrap();
        UserDB {
            email: self.email.to_string(),
            username: self.name.to_string(),
            dateofbirth: self.dateofbirth.clone(),
            contact: self.contact.clone(),
            password: hashed_password,
        }
    }
}

#[diesel(table_name = users)]
#[derive(Debug, Queryable, Insertable)]
pub struct UserDB {
    pub email: String,
    pub username: String,
    pub dateofbirth: Option<String>,
    pub contact: Option<String>,
    pub password: String,
}

impl UserDB {
    pub fn to_user(&self) -> User {
        User {
            email: self.email.to_string(),
            name: self.username.to_string(),
            dateofbirth: self.dateofbirth.clone(),
            contact: self.contact.clone(),
            password: self.password.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Queryable)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

pub fn register_user(user_data: User, conn: &mut DBPooledConnection) -> StatusResponse {
    use crate::schema::users::dsl::*;

    if !validate_email(&user_data.email) {
        return StatusResponse {
            status: "FAILED".to_string(),
            message: "Invalid Email".to_string(),
        };
    }

    if !validate_password(&user_data.password) {
        return StatusResponse {
            status: "FAILED".to_string(),
            message: "Password must be atleast 8 character long.".to_string(),
        };
    }

    let user_data = user_data.to_user_db();

    let _ = match users
        .filter(email.eq(&user_data.email))
        .first::<UserDB>(conn)
    {
        Ok(_) => {
            return StatusResponse {
                status: "FAILED".to_string(),
                message: "User Already Exist".to_string(),
            }
        }
        Err(NotFound) => match diesel::insert_into(users).values(user_data).execute(conn) {
            Ok(_) => {
                return StatusResponse {
                    status: "SUCCESS".to_string(),
                    message: "User Registered Successfully".to_string(),
                }
            }
            Err(err) => {
                return StatusResponse {
                    status: "FAILED".to_string(),
                    message: format!("Error while inserting data, {}", err),
                }
            }
        },
        Err(err) => {
            return StatusResponse {
                status: "FAILED".to_string(),
                message: format!("Error while checking for existing user: {}", err),
            }
        }
    };
}

pub fn login_user(login_data: LoginUser, conn: &mut DBPooledConnection) -> StatusResponse {
    use crate::schema::users::dsl::*;

    if !validate_email(&login_data.email) {
        return StatusResponse {
            status: "FAILED".to_string(),
            message: "Invalid Email".to_string(),
        };
    }

    let data = match users
        .filter(email.eq(&login_data.email))
        .select((email, password))
        .first::<LoginUser>(conn)
    {
        Ok(res) => res,
        Err(NotFound) => {
            return StatusResponse {
                status: "FAILED".to_string(),
                message: "No user found with given email".to_string(),
            };
        }
        Err(err) => {
            return StatusResponse {
                status: "FAILED".to_string(),
                message: format!("Error while checking existing database, {}", err),
            }
        }
    };

    let verify_auth = verify(&login_data.password, &data.password).unwrap();
    if !verify_auth {
        return StatusResponse {
            status: "FAILED".to_string(),
            message: "Invalid Password".to_string(),
        };
    }

    let ttl = env::var("ACCESS_TOKEN_MAX_AGE").expect("Failed to fetch env variable.");
    let ttl = ttl.parse::<i64>().unwrap();
    let private_key =
        env::var("BASE64_ACCESS_TOKEN_PRIVATE_KEY").expect("failed to fetch private key");
    let token = generate_jwt_token(login_data.email, ttl, private_key).unwrap();

    StatusResponse {
        status: "SUCCESS".to_string(),
        message: token.access_token.unwrap(),
    }
}

#[post("/register")]
async fn register(data: Json<User>, pool: Data<DBPool>) -> HttpResponse {
    let mut conn = pool.get().expect("Cannot establish connection to pool");

    let res = web::block(move || register_user(data.new_user(), &mut conn))
        .await
        .unwrap();

    HttpResponse::Ok().json(res)
}

#[post("/login")]
async fn login(data: Json<LoginUser>, pool: Data<DBPool>) -> HttpResponse {
    let mut conn = pool.get().expect("Cannot establish connection to pool");

    let temp_data = serde_json::to_string(&data).unwrap();
    let login_data: LoginUser = serde_json::from_str(&temp_data).unwrap();

    let res = web::block(move || login_user(login_data, &mut conn))
        .await
        .unwrap();

    HttpResponse::Ok().json(res)
}
