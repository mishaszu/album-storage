CREATE TABLE
  users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) NOT NULL UNIQUE,
    hash VARCHAR(255) NOT NULL,
    is_admin BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW (),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW ()
  );

-- initial password to change: dev_only_pwd
INSERT INTO
  users (id, email, hash, is_admin)
VALUES
  (
    'a74f9b43-8a49-4d97-8270-9879d37c600d',
    'root@test.com',
    '$argon2id$v=19$m=19456,t=2,p=1$AreaBODoNb1PVkrVYG47YQ$RqDZNg9uwWgRDFoeJkIED5RarIBPky6a0mvjr8sqVfs',
    true
  );

CREATE TABLE
  image (
    id Uuid PRIMARY KEY,
    title TEXT NOT NULL,
    original_full_title TEXT NOT NULL,
    description TEXT,
    path TEXT NOT NULL UNIQUE,
    width INT,
    height INT,
    is_uploaded BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP NOT NULL DEFAULT NOW (),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW ()
  );

CREATE TABLE
  album (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL UNIQUE,
    description TEXT,
    original_title TEXT NOT NULL,
    is_uploaded BOOLEAN NOT NULL DEFAULT FALSE,
    prev_image_id UUID REFERENCES image (id) ON DELETE NO ACTION,
    created_at TIMESTAMP NOT NULL DEFAULT NOW (),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW ()
  );

CREATE TABLE
  album_image (
    id UUID PRIMARY KEY,
    album_id UUID NOT NULL REFERENCES album (id) ON DELETE CASCADE,
    image_id UUID NOT NULL REFERENCES image (id) ON DELETE CASCADE,
    order_index INT NOT NULL,
    highlighted BOOLEAN NOT NULL DEFAULT FALSE,
    is_primary_album BOOLEAN NOT NULL
  );
