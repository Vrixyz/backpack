CREATE TABLE apps_items(
  app_id    int REFERENCES apps (id) ON UPDATE CASCADE ON DELETE CASCADE
, item_id    int REFERENCES items (id) ON UPDATE CASCADE ON DELETE CASCADE
-- explicit pk
, CONSTRAINT app_item_pkey PRIMARY KEY (app_id, item_id)
);