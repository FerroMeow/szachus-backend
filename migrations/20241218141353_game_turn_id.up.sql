-- Add up migration script here
ALTER TABLE game_turn
ADD COLUMN id INT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    ADD COLUMN turn_nr INT NOT NULL,
    ALTER COLUMN player_color
SET NOT NULL;