-- Add down migration script here
ALTER TABLE game_turn DROP COLUMN id,
    DROP COLUMN turn_nr,
    ALTER COLUMN player_color DROP NOT NULL;