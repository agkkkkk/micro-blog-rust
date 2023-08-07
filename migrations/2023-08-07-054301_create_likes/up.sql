-- Your SQL goes here

CREATE TABLE likes (
    id UUID PRIMARY KEY ,
    created_at TIMESTAMP DEFAULT NOW(),
    blog_id UUID NOT NULL
);
