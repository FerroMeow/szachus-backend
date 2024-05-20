-- Add migration script here
CREATE TABLE player (
  id int GENERATED ALWAYS AS IDENTITY,
  username varchar(63),
  score int,
  password_hash text,
  salt varchar(15)
);
CREATE TABLE game (
  id int GENERATED ALWAYS AS IDENTITY,
  started_at timestamp,
  ended_at timestamp,
  player_black int,
  player_white int
);
CREATE TABLE game_turn (
  game int,
  player_color smallint,
  tile_from varchar(2),
  tile_to varchar(2),
  pawn_moved varchar(12)
);