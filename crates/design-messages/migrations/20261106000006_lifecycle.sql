ALTER TABLE nodes ADD COLUMN role TEXT NOT NULL DEFAULT 'core';

ALTER TABLE nodes ADD COLUMN role_provenance TEXT NOT NULL DEFAULT 'init';

UPDATE nodes SET role_provenance = provenance;

ALTER TABLE nodes ADD COLUMN artifact TEXT;

ALTER TABLE nodes ADD COLUMN condition TEXT;

ALTER TABLE nodes ADD COLUMN reason TEXT;
