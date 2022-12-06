CREATE TABLE users_github(
   id serial PRIMARY KEY,
   login VARCHAR(50) NOT NULL,
   user_id int NOT NULL,
   FOREIGN KEY(user_id) REFERENCES users(id)
);