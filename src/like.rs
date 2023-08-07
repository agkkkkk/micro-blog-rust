use actix_web::web::{self, Data, Path};
use actix_web::{delete, get, post, HttpResponse};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use diesel::prelude::*;
use diesel::result::Error;
// use diesel::serialize::Result;
use diesel::{ExpressionMethods, Insertable, Queryable, RunQueryDsl};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

use super::schema::likes;
use crate::DBPool;
use crate::{response::Response, DBPooledConnection};

pub type Likes = Response<Like>;

#[derive(Debug, Deserialize, Serialize)]
pub struct Like {
    pub id: String,
    pub created_at: DateTime<Utc>,
}

impl Like {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            created_at: Utc::now(),
        }
    }

    pub fn to_db_likes(&self, blog_id: Uuid) -> LikeDB {
        let id = Uuid::from_str(self.id.as_ref()).unwrap();
        LikeDB {
            id: id,
            created_at: Utc::now().naive_utc(),
            blog_id: blog_id,
        }
    }
}

#[diesel(table_name = likes)]
#[derive(Queryable, Insertable)]

pub struct LikeDB {
    pub id: Uuid,
    pub created_at: NaiveDateTime,
    pub blog_id: Uuid,
}

impl LikeDB {
    pub fn to_like(&self) -> Like {
        Like {
            id: self.id.to_string(),
            created_at: Utc.from_utc_datetime(&self.created_at),
        }
    }
}

fn like_lists(_blog_id: Uuid, conn: &mut DBPooledConnection) -> Result<Likes, Error> {
    use crate::schema::likes::dsl::*;

    let _likes: Vec<LikeDB> = match likes
        .filter(blog_id.eq(_blog_id))
        .order(created_at.desc())
        .load::<LikeDB>(conn)
    {
        Ok(res) => res,
        Err(_) => vec![],
    };

    Ok(Likes {
        results: _likes
            .into_iter()
            .map(|l| l.to_like())
            .collect::<Vec<Like>>(),
    })
}

fn add_like(_blog_id: Uuid, conn: &mut DBPooledConnection) -> Result<Like, Error> {
    use crate::schema::likes::dsl::*;

    let like = Like::new();

    let _ = diesel::insert_into(likes)
        .values(like.to_db_likes(_blog_id))
        .execute(conn);

    Ok(like)
}

fn remove_like(_blog_id: Uuid, conn: &mut DBPooledConnection) -> Result<(), Error> {
    use crate::schema::likes::dsl::*;

    let _likes = like_lists(_blog_id, conn);

    let like = match &_likes {
        Ok(_likes) if !_likes.results.is_empty() => _likes.results.first(),
        _ => None,
    };

    if like.is_none() {
        return Ok(());
    }

    let like_id = Uuid::from_str(like.unwrap().id.as_str()).unwrap();

    let res = diesel::delete(likes.filter(id.eq(like_id))).execute(conn);

    match (res) {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

#[get("/blogs/{id}/likes")]
async fn list(path: Path<(String,)>, pool: Data<DBPool>) -> HttpResponse {
    let mut conn = pool.get().expect("Cannot connect to pool");
    let id: String = path.into_inner().0;
    let blog_id = Uuid::from_str(&id).unwrap();

    let likes = web::block(move || like_lists(blog_id, &mut conn))
        .await
        .unwrap()
        .unwrap();

    HttpResponse::Ok()
        .content_type("application/json")
        .json(likes)
}

#[post("/blogs/{id}/likes")]
async fn like_blog(path: Path<(String,)>, pool: Data<DBPool>) -> HttpResponse {
    let mut conn = pool.get().expect("Cannot connect to pool");
    let id = path.into_inner().0;
    let blog_id = Uuid::from_str(&id).unwrap();

    let likes = web::block(move || add_like(blog_id, &mut conn))
        .await
        .unwrap()
        .unwrap();

    HttpResponse::Created()
        .content_type("application/json")
        .json(likes)
}

#[delete("/blogs/{id}/likes")]
async fn dislike_blog(path: Path<(String,)>, pool: Data<DBPool>) -> HttpResponse {
    let mut conn = pool.get().expect("Cannot connect to pool");
    let id = path.into_inner().0;
    let blog_id = Uuid::from_str(&id).unwrap();

    let _ = web::block(move || remove_like(blog_id, &mut conn));

    HttpResponse::NoContent()
        .content_type("application/json")
        .await
        .unwrap()
}
