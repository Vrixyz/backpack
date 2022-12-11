CREATE TABLE apps_items(
  app_id    int REFERENCES apps (id) ON UPDATE CASCADE ON DELETE CASCADE
, item_id    int REFERENCES items (id) ON UPDATE CASCADE
/*
TODO: add rights of this app on this item:
- delete completely ? (should be only if last app with access)
- increase/decrease item amount
  - for users
  - for admins (game servers would be admins)
- add apps to this item, with which rights
*/
, CONSTRAINT app_item_pkey PRIMARY KEY (app_id, item_id)  -- explicit pk
);