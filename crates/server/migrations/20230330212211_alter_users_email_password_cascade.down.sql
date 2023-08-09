BEGIN;

-- drop the temporary constraint
ALTER TABLE users_email_password
DROP CONSTRAINT IF EXISTS users_email_password_user_id_fkey;

-- add the previous foreign key constraint without "on delete cascade"
ALTER TABLE users_email_password
ADD CONSTRAINT users_email_password_user_id_fkey
FOREIGN KEY (user_id) REFERENCES users(id);

COMMIT;