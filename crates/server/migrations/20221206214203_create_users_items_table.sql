CREATE TABLE users_items (
  user_id    int NOT NULL REFERENCES users (id) ON UPDATE CASCADE ON DELETE CASCADE
, item_id    int NOT NULL REFERENCES items (id) ON UPDATE CASCADE
, amount     int NOT NULL DEFAULT 0
, CONSTRAINT user_item_pkey PRIMARY KEY (user_id, item_id)  -- explicit pk
);