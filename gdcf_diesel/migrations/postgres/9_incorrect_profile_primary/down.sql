ALTER TABLE profile DROP CONSTRAINT profile_pkey;
ALTER TABLE profile ADD PRIMARY KEY (user_id);