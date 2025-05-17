CREATE TABLE admins (
    id integer PRIMARY KEY,
    username text CHECK (TRIM(username) <> '') NOT NULL UNIQUE,
    email text CHECK (TRIM(email) <> '') NOT NULL UNIQUE,
    password_phc text CHECK (TRIM(password_phc) <> '') NOT NULL UNIQUE
);

CREATE TABLE approval_records (
    id integer PRIMARY KEY, --autoincrements automatically
    date_added text NOT NULL,
    admin_id integer NOT NULL,
    FOREIGN KEY (admin_id) REFERENCES admins (id)
    ON DELETE RESTRICT
);

CREATE TABLE denial_records (
    id integer PRIMARY KEY, --autoincrements automatically
    date_added text NOT NULL,
    reason text CHECK (TRIM(reason) <> '') NOT NULL UNIQUE,
    admin_id integer NOT NULL,
    FOREIGN KEY (admin_id) REFERENCES admins (id)
    ON DELETE RESTRICT
);

CREATE TABLE sites (
    id integer PRIMARY KEY,
    root_url text CHECK (TRIM(root_url) <> '') NOT NULL UNIQUE,
    email text CHECK (TRIM(email) <> '') NOT NULL,
    approval_id integer UNIQUE,
    denial_id integer UNIQUE,
    CHECK (
    (approval_id IS NOT NULL AND denial_id IS NULL)
    or (approval_id IS NULL AND denial_id IS NOT NULL)
    or (approval_id IS NULL AND denial_id IS NULL)
    ),
    FOREIGN KEY (approval_id) REFERENCES approval_records (id)
    ON DELETE SET NULL,
    FOREIGN KEY (denial_id) REFERENCES denial_records (id)
    ON DELETE SET NULL
);

CREATE VIEW approved_sites
AS SELECT
    s.id AS site_id,
    s.root_url,
    s.email AS site_email,
    ar.date_added,
    a.id AS admin_id,
    a.username AS admin_username,
    a.email AS admin_email
FROM sites AS s
INNER JOIN approval_records AS ar ON s.approval_id = ar.id
LEFT JOIN admins AS a ON ar.admin_id = a.id;

CREATE VIEW denied_sites
AS SELECT
    s.id AS site_id,
    s.root_url,
    s.email AS site_email,
    dr.date_added,
    dr.reason,
    a.id AS admin_id,
    a.username AS admin_username,
    a.email AS admin_email
FROM sites AS s
INNER JOIN denial_records AS dr ON s.denial_id = dr.id
LEFT JOIN admins AS a ON dr.admin_id = a.id;

CREATE VIEW unapproved_sites
AS SELECT
    id,
    root_url,
    email
FROM sites
WHERE approval_id IS null AND denial_id IS null;
