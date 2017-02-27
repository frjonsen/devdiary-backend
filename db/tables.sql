CREATE EXTENSION IF NOT EXISTS pgcrypto;

DROP TABLE IF EXISTS Post CASCADE;
DROP TABLE IF EXISTS Project CASCADE;
DROP TABLE IF EXISTS Person CASCADE;

CREATE TABLE IF NOT EXISTS Person(
  id uuid DEFAULT gen_random_uuid() PRIMARY KEY,
  username text NOT NULL UNIQUE CHECK (LENGTH(username) > 2),
  fullname text,
  password char (60) NOT NULL CHECK (LENGTH(password) = 60)
);

CREATE TABLE IF NOT EXISTS Project(
  id uuid DEFAULT gen_random_uuid() PRIMARY KEY,
  name text,
  description text,
  creator uuid NOT NULL REFERENCES Person(id),
  created timestamptz DEFAULT now() NOT NULL
);

CREATE TABLE IF NOT EXISTS Post(
  id uuid DEFAULT gen_random_uuid() PRIMARY KEY,
  title text,
  content text,
  author uuid NOT NULL REFERENCES Person(id),
  project uuid NOT NULL REFERENCES Project(id),
  created timestamptz DEFAULT now() NOT NULL,
  shareurl text DEFAULT NULL
);