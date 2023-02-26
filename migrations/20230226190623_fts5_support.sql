CREATE VIRTUAL TABLE statements_fts USING fts5(id UNINDEXED, text);

CREATE TRIGGER statements_ai AFTER INSERT ON statements
BEGIN
  INSERT INTO statements_fts (id, text) VALUES (new.id, new.text);
END;

-- Migrate existing data into fts table
INSERT INTO statements_fts SELECT id, text FROM statements;
