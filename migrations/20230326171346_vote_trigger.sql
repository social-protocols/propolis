CREATE TRIGGER vote_history_ai AFTER INSERT ON vote_history
BEGIN
  INSERT INTO votes (statement_id, user_id, vote)
              VALUES (new.statement_id, new.user_id, new.vote)
              on CONFLICT (statement_id, user_id)
              do UPDATE SET vote = new.vote;
END;
