CREATE TABLE users_github(
   /*
   Github exact data.
   */
   id INT PRIMARY KEY,
   login TEXT NOT NULL,
   /*
   Reference to the user connected with that github account.
   */
   user_id int NOT NULL,
   FOREIGN KEY(user_id) REFERENCES users(id)
);