CREATE TABLE apps_admins(
  user_id    int NOT NULL REFERENCES users (id) ON UPDATE CASCADE ON DELETE CASCADE
, app_id    int NOT NULL REFERENCES apps (id) ON UPDATE CASCADE ON DELETE CASCADE
, CONSTRAINT user_admin_pkey PRIMARY KEY (user_id, app_id)  -- explicit pk
);