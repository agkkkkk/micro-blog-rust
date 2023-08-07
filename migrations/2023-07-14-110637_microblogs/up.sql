-- Your SQL goes here

CREATE TABLE microblogs (
    id UUID PRIMARY KEY ,
    blog_message TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE EXTENSION "uuid-ossp";