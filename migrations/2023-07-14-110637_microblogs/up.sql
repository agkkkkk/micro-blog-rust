-- Your SQL goes here

CREATE TABLE microblogs (
    id UUID PRIMARY KEY ,
    blog_message TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE EXTENSION "uuid-ossp";