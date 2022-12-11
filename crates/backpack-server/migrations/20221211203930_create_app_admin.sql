CREATE TABLE apps_admins(
  user_id    int REFERENCES users (id) ON UPDATE CASCADE ON DELETE CASCADE
, app_id    int REFERENCES apps (id) ON UPDATE CASCADE
, CONSTRAINT user_admin_pkey PRIMARY KEY (user_id, app_id)  -- explicit pk
);