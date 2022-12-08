CREATE TABLE users_github(
   id serial PRIMARY KEY,
   login TEXT NOT NULL,
   user_id int NOT NULL,
   FOREIGN KEY(user_id) REFERENCES users(id)
);