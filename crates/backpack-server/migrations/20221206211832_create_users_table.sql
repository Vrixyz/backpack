CREATE TABLE users(
   id serial PRIMARY KEY,
   name VARCHAR(50) NOT NULL,
   created_at TIMESTAMP NOT NULL DEFAULT NOW()
);