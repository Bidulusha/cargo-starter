-- Создаем пользователя если не существует
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'messenger_user') THEN
        CREATE USER messenger_user WITH PASSWORD 'Ra66RDR8?4+Bqu4p8Vb4Sk3bp8J';
    END IF;
END
$$;

  

-- Создаем базу данных если не существует
SELECT 'CREATE DATABASE messenger_db'
    WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'messenger_db')\gexec

-- Подключаемся к базе
\c messenger_db

-- Даем права на схему public
GRANT ALL ON SCHEMA public TO messenger_user;
GRANT ALL PRIVILEGES ON DATABASE messenger_db TO messenger_user;

-- Создаем ENUM типы с проверкой существования через DO
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'MessageContent') THEN
        CREATE TYPE MessageContent AS (
        answer_to       integer,
        forwarded_from  integer,
        text_content    text,
        photos_content  text[],
        files           text[]
        );
    END IF;
END
$$;

-- Создаем таблицы
create table if not exists public.users_info (
    id integer NOT NULL GENERATED ALWAYS AS IDENTITY (INCREMENT 1 START 1 MINVALUE 1 MAXVALUE 2147483647 CACHE 1) primary key,
    email varchar(64) NOT NULL,
    login varchar(32) NOT NULL,
    password varchar(128) NOT NULL,
    avatar text NOT NULL
);

create table if not exists public.chats_info(
    id integer NOT NULL GENERATED ALWAYS AS IDENTITY (INCREMENT 1 START 1 MINVALUE 1 MAXVALUE 2147483647 CACHE 1) primary key,
    avatar text NOT NULL,
    chat_name varchar(128) NOT NULL,
    members_id integer[] NOT NULL
);

-- create table if not exists public.active_sessions (
--     id integer NOT NULL GENERATED ALWAYS AS IDENTITY (INCREMENT 1 START 1 MINVALUE 1 MAXVALUE 2147483647 CACHE 1),
--     user_id bigint NOT NULL,
--     token text NOT NULL
-- );

-- Даем права на таблицу и последовательности
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO messenger_user;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO messenger_user;