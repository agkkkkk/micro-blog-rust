-- Your SQL goes here

CREATE TABLE users (
    email VARCHAR(50) PRIMARY KEY,
    username VARCHAR(30) NOT NULL,
    dateofbirth VARCHAR(10),
    contact VARCHAR(10),
    password TEXT NOT NULL
)