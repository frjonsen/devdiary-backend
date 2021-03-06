﻿CREATE EXTENSION IF NOT EXISTS pgcrypto;

DROP TABLE IF EXISTS Session CASCADE;
DROP TABLE IF EXISTS Post CASCADE;
DROP TABLE IF EXISTS Project CASCADE;
DROP TABLE IF EXISTS Person CASCADE;
DROP TABLE IF EXISTS GithubUser;
DROP TABLE IF EXISTS LocalUser;

CREATE TABLE IF NOT EXISTS Person(
  id uuid DEFAULT gen_random_uuid() PRIMARY KEY,
  username text NOT NULL UNIQUE CHECK (LENGTH(username) > 2),
  fullname text
);

CREATE TABLE IF NOT EXISTS LocalUser(
  id uuid PRIMARY KEY REFERENCES Person(id),
  password char (60) NOT NULL CHECK (LENGTH(password) = 60)
);

CREATE TABLE IF NOT EXISTS GithubUser(
  id uuid PRIMARY KEY REFERENCES Person(id),
  github_access_token Text UNIQUE NOT NULL
);

CREATE TABLE IF NOT EXISTS Project(
  id uuid DEFAULT gen_random_uuid() PRIMARY KEY,
  name text,
  description text,
  creator uuid NOT NULL REFERENCES Person(id),
  created timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE IF NOT EXISTS Post(
  id uuid DEFAULT gen_random_uuid() PRIMARY KEY,
  title text,
  content text,
  author uuid NOT NULL REFERENCES Person(id),
  project uuid NOT NULL REFERENCES Project(id),
  created timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL,
  shareurl text DEFAULT NULL
);

CREATE TABLE IF NOT EXISTS Session(
  token Text PRIMARY KEY,
  person_id uuid NOT NULL REFERENCES Person(id),
  created timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL,
  access timestamptz DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX idx_session_person ON Session(person_id);