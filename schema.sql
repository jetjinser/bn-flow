DROP TABLE IF EXISTS bn_trigger ;

CREATE TABLE IF NOT EXISTS bn_trigger (
  id serial PRIMARY KEY,
  address TEXT NOT NULL,
  flow_id TEXT NOT NULL,
  flows_user TEXT NOT NULL
);
