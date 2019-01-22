-- Your SQL goes here
CREATE TABLE guilds (
    id BIGINT PRIMARY KEY,
    prefix TEXT,
    youtube_results INTEGER DEFAULT 5,
    imgur_album_id VARCHAR
)