DROP FUNCTION IF EXISTS create_user(text, text, text);
DROP FUNCTION IF EXISTS change_password(uuid, text);
DROP FUNCTION IF EXISTS authenticate_user(text, text);
DROP FUNCTION IF EXISTS generate_session_token();
DROP FUNCTION IF EXISTS create_new_session(uuid);
DROP FUNCTION IF EXISTS remove_session(text);
DROP FUNCTION IF EXISTS access_session_token(text, text);
DROP FUNCTION IF EXISTS clear_all_users_sessions(uuid);
DELETE FROM Session;
DELETE FROM Project;
DELETE FROM Post;
DELETE FROM Person;

CREATE FUNCTION create_user (uname text, pwd text, fname text = NULL)
RETURNS SETOF Person AS
$$
BEGIN
IF LENGTH(pwd) < 5 THEN
  RAISE 'Password too short, must be at least 5 characters' using ERRCODE='invalid_parameter_value';
ELSIF LENGTH(pwd) > 72 THEN
  RAISE 'Password too long, must be shorter than 72 characters' using ERRCODE='invalid_parameter_value';
END IF;

  RETURN QUERY INSERT INTO Person(username, fullname, password) VALUES (uname, fname, crypt("pwd", gen_salt('bf', 10))) RETURNING *;
END
$$ LANGUAGE 'plpgsql';

CREATE FUNCTION authenticate_user(uname text, pwd text)
RETURNS Person AS
$$
DECLARE
  _user Person;
BEGIN
  SELECT * FROM Person WHERE username=uname AND (password=crypt(pwd, password)) INTO _user;
  RETURN _user;
END
$$ LANGUAGE 'plpgsql';

CREATE FUNCTION change_password(id uuid, pwd text)
RETURNS BOOLEAN AS
$$
BEGIN
  IF LENGTH(pwd) < 5 THEN
    RAISE 'Password too short, must be at least 5 characters' using ERRCODE='invalid_parameter_value';
  ELSIF LENGTH(pwd) > 72 THEN
    RAISE 'Password too long, must be shorter than 72 characters' using ERRCODE='invalid_parameter_value';
  END IF;

  UPDATE Person SET password = (crypt("pwd", gen_salt('bf', 10)));

  IF FOUND THEN
    RETURN TRUE;
  ELSE
    RETURN FALSE;
  END IF;
END
$$ LANGUAGE 'plpgsql';

CREATE FUNCTION generate_session_token()
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

CREATE FUNCTION create_new_session(_id uuid)
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

CREATE FUNCTION remove_session(_token Text)
RETURNS BOOLEAN AS
$$
BEGIN
  IF (SELECT NOT EXISTS (SELECT Session.token FROM Session WHERE Session.token = _token)) THEN
    RAISE 'No session with token "_token"' using ERRCODE='invalid_parameter_value';
  END IF;
  
END $$ LANGUAGE 'plpgsql';

CREATE FUNCTION clear_all_users_sessions(id uuid)
RETURNS VOID AS
$$
BEGIN
  DELETE FROM Session WHERE person_id = id;
END $$ LANGUAGE 'plpgsql';

CREATE FUNCTION access_session_token(_token Text, token_valid_duration Text = NULL)
RETURNS Person AS
$$
DECLARE
  _session Session;
  _user Person;
  _validuntil timestamptz;
BEGIN
  SELECT * FROM Session WHERE Session.token = _token INTO _session;
  IF NOT FOUND THEN
    RETURN NULL;
  END IF;
  SELECT * FROM Person WHERE Person.id = _session.person_id INTO _user;

  /* Caller requested check to see if session is still valid */
  IF token_valid_duration IS NOT NULL THEN
    SELECT (_session.access + token_valid_duration::INTERVAL) INTO _validuntil;
    IF (_validuntil < CURRENT_TIMESTAMP) THEN
      /* If token has expired, delete it from the database */
      DELETE FROM Session WHERE Session.token = _token;
      RETURN NULL;
    END IF;
  END IF;
  -- UPDATE Session SET access = (SELECT now()) WHERE token = _token;
  RETURN _user;
END $$ LANGUAGE 'plpgsql';