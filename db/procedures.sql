DROP FUNCTION IF EXISTS add_user(text, text, text);
DROP FUNCTION IF EXISTS change_password(uuid, text);
DROP FUNCTION IF EXISTS authenticate_user(text, text);
DELETE FROM Project;
DELETE FROM Person;

CREATE FUNCTION add_user (uname text, pwd text, fname text = NULL)
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
RETURNS SETOF Person AS
$$
DECLARE
auth_user Person;
BEGIN
  RETURN QUERY SELECT * FROM Person WHERE username=uname AND (password=crypt(pwd, password));
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

SELECT * FROM add_user('som', 'newpassword');