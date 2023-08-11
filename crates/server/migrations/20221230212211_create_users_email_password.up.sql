CREATE TABLE users_email_password(
   id serial PRIMARY KEY,
   email varchar(254) NOT NULL UNIQUE,
   password_hash text NOT NULL,
   is_verified boolean NOT NULL,
   /*
   Reference to the user connected with that email/password.
   */
   user_id int NOT NULL,
   FOREIGN KEY(user_id) REFERENCES users(id) ON UPDATE CASCADE ON DELETE CASCADE
);