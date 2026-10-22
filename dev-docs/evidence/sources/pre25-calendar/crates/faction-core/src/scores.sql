CREATE TABLE game_scores (
    member_id INTEGER NOT NULL REFERENCES members(id),
    game TEXT NOT NULL CHECK(length(game) BETWEEN 1 AND 32),
    best INTEGER NOT NULL CHECK(best BETWEEN 0 AND 1000000000),
    plays INTEGER NOT NULL CHECK(plays BETWEEN 1 AND 1000000000),
    PRIMARY KEY(member_id,game)
);
CREATE INDEX game_leaders ON game_scores(game,best DESC,member_id);
