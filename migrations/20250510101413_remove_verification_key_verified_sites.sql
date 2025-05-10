drop view verified_sites;

CREATE VIEW verified_sites
AS SELECT
    s.root_url,
    s.email,
    v.date_added,
    a.username AS admin_username,
    a.email AS admin_email
FROM sites AS s
INNER JOIN verification_details AS v ON s.verification_id = v.id
LEFT JOIN admins AS a ON v.admin_id = a.id;
