CREATE TABLE refresh_tokens (
    id SERIAL PRIMARY KEY,
    refresh_token VARCHAR(255) NOT NULL,
    user_id INT NOT NULL REFERENCES users (id) ON UPDATE CASCADE ON DELETE CASCADE,
    expiration_date TIMESTAMP NOT NULL,
    -- TODO: #17 We don't want to immediately delete a refresh token, so we can detect refresh token reuse, maybe due to malicious usage.
    revoked BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL
);