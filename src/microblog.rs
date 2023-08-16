use actix_web::{
    delete, get, post,
    web::{self, Data, Json, Path},
    HttpResponse,
};
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use diesel::query_dsl::methods::{FilterDsl, LimitDsl, OrderDsl};
// use diesel::query_dsl::RunQueryDsl;
use diesel::result::Error;
use diesel::{ExpressionMethods, Insertable, Queryable, RunQueryDsl};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use uuid::Uuid;

use crate::like::{like_lists, Like};
use crate::response::Response;
use crate::{DBPool, DBPooledConnection};

use super::schema::microblogs;

pub type MicroBlogs = Response<MicroBlog>;

#[derive(Debug, Deserialize, Serialize)]
pub struct MicroBlog {
    pub id: String,
    pub blog_message: String,
    pub created_at: DateTime<Utc>,
    pub likes: Vec<Like>,
}

impl MicroBlog {
    pub fn new(blog: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            blog_message: blog,
            created_at: Utc::now(),
            likes: vec![],
        }
    }

    pub fn to_db_microblog(&self) -> MicroBlogDB {
        let id = Uuid::from_str(self.id.as_ref()).unwrap();
        MicroBlogDB {
            id: id,
            blog_message: self.blog_message.clone(),
            created_at: Utc::now().naive_utc(),
        }
    }

    pub fn add_likes(&self, likes: Vec<Like>) -> Self {
        MicroBlog {
            id: self.id.clone(),
            blog_message: self.blog_message.clone(),
            created_at: self.created_at,
            likes: likes,
        }
    }
}

#[diesel(table_name = microblogs)]
#[derive(Queryable, Insertable)]
pub struct MicroBlogDB {
    pub id: Uuid,
    pub blog_message: String,
    pub created_at: NaiveDateTime,
}

impl MicroBlogDB {
    fn to_blog(&self) -> MicroBlog {
        MicroBlog {
            id: self.id.to_string(),
            blog_message: self.blog_message.to_string(),
            created_at: Utc.from_utc_datetime(&self.created_at),
            likes: vec![],
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BlogRequest {
    pub blog: Option<String>,
}

impl BlogRequest {
    pub fn new_blog_request(&self) -> Option<MicroBlog> {
        match &self.blog {
            Some(a) => Some(MicroBlog::new(a.to_string())),
            None => None,
        }
    }
}

pub fn list_blogs(total_blogs: i64, conn: &mut DBPooledConnection) -> Result<MicroBlogs, Error> {
    use crate::schema::microblogs::dsl::*;

    let _blogs = match microblogs
        .order(created_at.desc())
        .limit(total_blogs)
        .load::<MicroBlogDB>(conn)
    {
        Ok(res) => res,
        Err(_) => vec![],
    };

    Ok(MicroBlogs {
        results: _blogs
            .into_iter()
            .map(|t| t.to_blog())
            .collect::<Vec<MicroBlog>>(),
    })
}

fn create_blog(blog_msg: MicroBlog, conn: &mut DBPooledConnection) -> Result<MicroBlog, ()> {
    use crate::schema::microblogs::dsl::*;

    let blog_db = blog_msg.to_db_microblog();
    let _ = diesel::insert_into(microblogs)
        .values(&blog_db)
        .execute(conn);

    Ok(blog_db.to_blog())
}

fn get_blog_by_uuid(_id: Uuid, conn: &mut DBPooledConnection) -> Result<MicroBlog, Error> {
    use crate::schema::microblogs::dsl::*;

    let blog = microblogs.filter(id.eq(_id)).load::<MicroBlogDB>(conn);

    match blog {
        Ok(b) => match b.first() {
            Some(first) => Ok(first.to_blog()),
            _ => Err(Error::NotFound),
        },
        Err(err) => Err(err),
    }
}

fn delete_blog_by_uuid(_id: Uuid, conn: &mut DBPooledConnection) -> Result<(), Error> {
    use crate::schema::microblogs::dsl::*;

    let blog = diesel::delete(microblogs.filter(id.eq(_id))).execute(conn);

    match blog {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

#[get("/blogs")]
async fn blogs(pool: Data<DBPool>) -> HttpResponse {
    let mut conn = pool.get().expect("Cannot connect to pool");
    let mut blogss = web::block(move || list_blogs(50, &mut conn))
        .await
        .unwrap()
        .unwrap();

    let mut conn = pool.get().expect("Cannot connect to pool");

    let blogs = MicroBlogs {
        results: blogss
            .results
            .iter_mut()
            .map(|b| {
                let likes = like_lists(Uuid::from_str(b.id.as_str()).unwrap(), &mut conn).unwrap();
                b.add_likes(likes.results)
            })
            .collect::<Vec<MicroBlog>>(),
    };

    HttpResponse::Ok()
        .content_type("application/json")
        .json(blogs)
}

#[post("/blogs")]
async fn create_blogs(blog: Json<BlogRequest>, pool: Data<DBPool>) -> HttpResponse {
    let mut conn = pool.get().expect("Cannot connect to DB pool");

    let blog = web::block(move || create_blog(blog.new_blog_request().unwrap(), &mut conn)).await;

    match blog {
        Ok(blog) => {
            println!("{:?}", blog);
            HttpResponse::Created()
                .content_type("application/json")
                .json(blog)
        }
        _ => HttpResponse::NoContent().await.unwrap(),
    }
}

#[get("/blogs/{id}")]
async fn get_blog(path: Path<(String,)>, pool: Data<DBPool>) -> HttpResponse {
    let mut conn = pool.get().expect("Cannot connect to DB pool");

    let (id,): (String,) = path.into_inner();
    // println!("{}", id);

    let blog =
        web::block(move || get_blog_by_uuid(Uuid::from_str(id.as_str()).unwrap(), &mut conn))
            .await
            .unwrap();

    println!("{:?}", blog);

    match blog {
        Ok(blog) => HttpResponse::Ok()
            .content_type("application/json")
            .json(blog),

        _ => HttpResponse::NoContent()
            .content_type("application/json")
            .await
            .unwrap(),
    }
}

#[delete("/blogs/{id}")]
async fn delete_blog(path: Path<(String,)>, pool: Data<DBPool>) -> HttpResponse {
    let mut conn = pool.get().expect("Cannot connect to DB Pool");

    let (id,) = path.into_inner();
    println!("{}", id);

    let _ =
        web::block(move || delete_blog_by_uuid(Uuid::from_str(id.as_str()).unwrap(), &mut conn))
            .await;

    HttpResponse::NoContent()
        .content_type("application/json")
        .await
        .unwrap()
}
