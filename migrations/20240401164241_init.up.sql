-- Add migration script here
CREATE TABLE player (
  id int GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  username varchar(63),
  score int DEFAULT 0 CHECK (score >= 0),
  password_hash text,
  salt varchar(15)
);
CREATE TABLE game (
  id int GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  started_at timestamp,
  ended_at timestamp,
  player_black int REFERENCES player,
  player_white int REFERENCES player
);
CREATE TABLE game_turn (
  game int REFERENCES game,
  player_color smallint CHECK(
    player_color >= 0
    OR player_color <= 1
  ),
  tile_from varchar(2),
  tile_to varchar(2),
  pawn_moved varchar(12)
);