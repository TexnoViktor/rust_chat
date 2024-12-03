-- migrations/20240103000000_create_tables.sql
DROP TABLE IF EXISTS messages;
DROP TABLE IF EXISTS users;

CREATE TABLE users (
                       id SERIAL PRIMARY KEY,
                       username VARCHAR(255) UNIQUE NOT NULL,
                       password VARCHAR(255) NOT NULL,
                       created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE messages (
                          id SERIAL PRIMARY KEY,
                          from_user_id INTEGER NOT NULL REFERENCES users(id),
                          to_user_id INTEGER NOT NULL REFERENCES users(id),
                          content TEXT NOT NULL,
                          message_type VARCHAR(50) NOT NULL,
                          file_path VARCHAR(255),
                          created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_messages_users ON messages(from_user_id, to_user_id);
CREATE INDEX idx_messages_timestamp ON messages(created_at);