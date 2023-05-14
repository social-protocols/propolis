drop trigger statements_ad;
CREATE TRIGGER statements_ad AFTER DELETE ON statements BEGIN
  -- update search index
  DELETE FROM statements_fts WHERE id = old.id;
END;

CREATE TRIGGER statements_au AFTER UPDATE ON statements BEGIN
  -- update search index
  DELETE FROM statements_fts WHERE id = old.id;
  insert into statements_fts(id, text) values (new.id, new.text);
END;
