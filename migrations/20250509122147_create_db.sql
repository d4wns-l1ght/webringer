CREATE TABLE admins (
    id integer PRIMARY KEY,
    username text CHECK (TRIM(email) <> '') NOT NULL UNIQUE,
    email text CHECK (TRIM(email) <> '') NOT NULL UNIQUE,
    password_hash text CHECK (TRIM(email) <> '') NOT NULL,
    password_salt text CHECK (TRIM(email) <> '') NOT NULL UNIQUE
);

CREATE TABLE verification_details (
    id integer PRIMARY KEY, --autoincrements automatically
    date_added text NOT NULL,
    verification_key text CHECK (TRIM(verification_key) <> '') NOT NULL,
    admin_id integer NOT NULL,
    FOREIGN KEY (admin_id) REFERENCES admins (id)
    ON DELETE RESTRICT
);

CREATE TABLE sites (
    root_url text CHECK (TRIM(root_url) <> '') NOT NULL PRIMARY KEY,
    email text CHECK (TRIM(email) <> '') NOT NULL,
    verification_id integer UNIQUE,
    FOREIGN KEY (verification_id) REFERENCES verification_details (id)
    ON DELETE SET NULL
);

CREATE VIEW verified_sites
AS SELECT
    s.root_url,
    s.email,
    v.date_added,
    v.verification_key,
    a.username AS admin_username,
    a.email AS admin_email
FROM sites AS s
INNER JOIN verification_details AS v ON s.verification_id = v.id
LEFT JOIN admins AS a ON v.admin_id = a.id;

CREATE VIEW unverified_sites
AS SELECT
    root_url,
    email
FROM sites
WHERE verification_id IS null;
