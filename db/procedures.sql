DELETE FROM GithubUser;
DELETE FROM LocalUser;
DELETE FROM Session;
DELETE FROM Project;
DELETE FROM Post;
DELETE FROM Person;

CREATE OR REPLACE FUNCTION create_local_user (uname text, pwd text, fname text = NULL)
RETURNS Person AS
$$
DECLARE
  _new_user Person;
BEGIN
IF LENGTH(pwd) < 5 THEN
  RAISE 'Password too short, must be at least 5 characters' using ERRCODE='invalid_parameter_value';
ELSIF LENGTH(pwd) > 72 THEN
  RAISE 'Password too long, must be shorter than 72 characters' using ERRCODE='invalid_parameter_value';
END IF;

  INSERT INTO Person(username, fullname) VALUES (uname, fname) RETURNING Person.id, Person.username, Person.fullname INTO _new_user;
  INSERT INTO LocalUser(id, password) VALUES (_new_user.id, crypt("pwd", gen_salt('bf', 10)));
  RETURN _new_user;
END
$$ LANGUAGE 'plpgsql';

CREATE OR REPLACE FUNCTION create_github_user(uname text, _github_access_token text, fname text = NULL)
RETURNS Person AS
$$
DECLARE
  _new_user Person;
BEGIN
  INSERT INTO Person(username, fullname) VALUES (uname, fname) RETURNING Person.id, Person.username, Person.fullname INTO _new_user;
  INSERT INTO GithubUser(id, github_access_token) VALUES (_new_user.id, _github_access_token);
  RETURN _new_user;
END $$ LANGUAGE 'plpgsql';

CREATE OR REPLACE FUNCTION authenticate_github_user(_github_access_token TEXT)
RETURNS SETOF Person AS
$$
BEGIN
  RETURN QUERY SELECT * FROM Person WHERE Person.id = (SELECT GithubUser.id FROM GithubUser WHERE github_access_token = _github_access_token);
END
$$ LANGUAGE 'plpgsql';

CREATE OR REPLACE FUNCTION authenticate_local_user(uname text, pwd text)
RETURNS SETOF Person AS
$$
BEGIN
  RETURN QUERY SELECT * FROM Person WHERE username=uname AND (EXISTS(SELECT id FROM LocalUser WHERE Person.id = LocalUser.id AND password=crypt(pwd, password)));
END
$$ LANGUAGE 'plpgsql';

CREATE OR REPLACE FUNCTION get_user(_id uuid = NULL, _username text = NULL)
RETURNS SETOF Person AS
$$
BEGIN
  IF _id IS NOT NULL AND _username IS NOT NULL THEN
    RETURN QUERY SELECT * FROM Person WHERE Person.id = _id AND Person.username = _username;
  ELSIF _id IS NOT NULL THEN
    RETURN QUERY SELECT * FROM Person WHERE Person.id = _id;
  ELSIF _username IS NOT NULL THEN
    RETURN QUERY SELECT * FROM Person WHERE Person.username = _username;
  ELSE
    RAISE 'Must specify at least one parameter' using ERRCODE='invalid_parameter_value';
  END IF;
END $$ LANGUAGE 'plpgsql';

CREATE OR REPLACE FUNCTION change_password(_id uuid, pwd text)
RETURNS BOOLEAN AS
$$
BEGIN
  UPDATE LocalUser SET password = (crypt("pwd", gen_salt('bf', 10))) WHERE Person.id = _id;

  IF FOUND THEN
    RETURN TRUE;
  ELSE
    RETURN FALSE;
  END IF;
END
$$ LANGUAGE 'plpgsql';

CREATE OR REPLACE FUNCTION generate_session_token()
RETURNS Text AS 
$$
DECLARE
  token text;
BEGIN
  SELECT (
    SELECT string_agg(x, '') 
    FROM (
      SELECT chr(ascii('!') + floor(random() * 93)::integer)
      FROM generate_series(1, 80)
    ) AS y(x)
  ) INTO token;
  RETURN token;
END $$ LANGUAGE 'plpgsql';

CREATE OR REPLACE FUNCTION create_new_session(_id uuid)
RETURNS Text AS
$$
DECLARE
  _token text;
BEGIN
  IF (SELECT NOT EXISTS(SELECT Person.id FROM Person WHERE Person.id = _id)) THEN
    RAISE 'User does not exist' using ERRCODE='invalid_parameter_value';
  END IF;
  
  LOOP
    SELECT generate_session_token() INTO _token;
    IF (SELECT NOT EXISTS(SELECT Session.token FROM Session WHERE Session.token = _token)) THEN
      EXIT;
    END IF;
  END LOOP;

  INSERT INTO Session(token, person_id) VALUES (_token, _id);
  
  RETURN (SELECT _token);
END $$ LANGUAGE 'plpgsql';

CREATE OR REPLACE FUNCTION remove_session(_token Text)
RETURNS BOOLEAN AS
$$
BEGIN
  IF (SELECT NOT EXISTS (SELECT Session.token FROM Session WHERE Session.token = _token)) THEN
    RAISE 'No session with token "_token"' using ERRCODE='invalid_parameter_value';
  END IF;
  
END $$ LANGUAGE 'plpgsql';

CREATE OR REPLACE FUNCTION clear_all_users_sessions(id uuid)
RETURNS VOID AS
$$
BEGIN
  DELETE FROM Session WHERE person_id = id;
END $$ LANGUAGE 'plpgsql';

CREATE OR REPLACE FUNCTION access_session_token(_token Text, update_access BOOLEAN = TRUE, token_valid_duration Text = NULL)
RETURNS TABLE(id uuid, username Text, fullname text) AS
$$
DECLARE
  _session Session;
  _user Person;
  _validuntil timestamptz;
BEGIN
  SELECT * FROM Session WHERE Session.token = _token INTO _session;
  IF NOT FOUND THEN
    RETURN QUERY SELECT(NULL);
  END IF;

  /* Caller requested check to see if session is still valid */
  IF token_valid_duration IS NOT NULL THEN
    SELECT (_session.access + token_valid_duration::INTERVAL) INTO _validuntil;
    IF (_validuntil < CURRENT_TIMESTAMP) THEN
      /* If token has expired, delete it from the database */
      DELETE FROM Session WHERE Session.token = _token;
      RETURN QUERY SELECT(NULL);
    END IF;
  END IF;

  /* Caller requested the access lifetime be updated */
  IF update_access THEN
    UPDATE Session SET access = CURRENT_TIMESTAMP WHERE token = _token;
  END IF;
  
  RETURN QUERY SELECT Person.id, Person.username, Person.fullname FROM Person WHERE Person.id = _session.person_id;
END $$ LANGUAGE 'plpgsql';

SELECT * FROM create_github_user('ausername', 'apassword');
/*SELECT create_new_session((SELECT id FROM Person));
SELECT * FROM access_session_token((SELECT token FROM Session));*/
SELECT * FROM get_user(NULL, 'ausername');
SELECT * FROM GithubUser;