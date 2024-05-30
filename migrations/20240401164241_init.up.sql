-- Add migration script here
CREATE TABLE player (
  id int GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  username varchar(63) NOT NULL UNIQUE,
  score int DEFAULT 0 CHECK (score >= 0) NOT NULL,
  password_hash text NOT NULL,
  salt varchar(63) NOT NULL
);
CREATE TABLE game (
  id int GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  started_at timestamp NOT NULL,
  ended_at timestamp NOT NULL,
  player_black int REFERENCES player,
  player_white int REFERENCES player
);
CREATE TABLE game_turn (
  game int REFERENCES game,
  player_color smallint CHECK(
    player_color >= 0
    OR player_color <= 1
  ),
  tile_from varchar(2) NOT NULL,
  tile_to varchar(2) NOT NULL,
  pawn_moved varchar(12) NOT NULL
);