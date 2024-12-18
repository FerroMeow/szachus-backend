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
  ended_at timestamp,
  player_black int REFERENCES player NOT NULL,
  player_white int REFERENCES player NOT NULL
);
CREATE TABLE game_turn (
  id int GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
  turn_nr INT NOT NULL,
  game int NOT NULL,
  player_color varchar(5) NOT NULL,
  tile_from varchar(2) NOT NULL,
  tile_to varchar(2) NOT NULL,
  pawn_moved varchar(12) NOT NULL,
  FOREIGN KEY (game) REFERENCES game ON DELETE CASCADE ON UPDATE CASCADE
);