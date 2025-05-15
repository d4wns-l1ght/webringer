DROP VIEW unverified_sites;

CREATE VIEW unverified_sites
AS SELECT
    id,
    root_url,
    email
FROM sites
WHERE verification_id IS null;
