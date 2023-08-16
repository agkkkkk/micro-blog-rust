use crate::{
    response::StatusResponse,
    validation::{validate_email, validate_password},
    DBPool, DBPooledConnection,
};

use super::schema::users;
use actix_web::{
    post,
    web::{self, Data, Json},
    HttpResponse,
};
use bcrypt::{hash, DEFAULT_COST};
use diesel::{
    result::Error::NotFound, ExpressionMethods, Insertable, QueryDsl, Queryable, RunQueryDsl,
};
use serde::{Deserialize, Serialize};

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

#[post("/register")]
async fn register(data: Json<User>, pool: Data<DBPool>) -> HttpResponse {
    let mut conn = pool.get().expect("Cannot establish connection to pool");

    let res = web::block(move || register_user(data.new_user(), &mut conn))
        .await
        .unwrap();

    HttpResponse::Ok().json(res)
}
