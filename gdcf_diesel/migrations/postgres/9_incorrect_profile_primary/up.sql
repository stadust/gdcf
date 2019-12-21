-- For profile we know that the account_id exists, because profiles can only be retrieved by account_id
ALTER TABLE profile DROP CONSTRAINT profile_pkey;
ALTER TABLE profile ADD PRIMARY KEY (account_id);