BEGIN;

-- drop the temporary constraint
ALTER TABLE users_email_password
DROP CONSTRAINT users_email_password_user_id_fkey;

-- add the "on delete cascade" constraint
ALTER TABLE users_email_password
ADD CONSTRAINT users_email_password_user_id_fkey
FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

COMMIT;