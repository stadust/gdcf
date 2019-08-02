CREATE OR REPLACE FUNCTION pg_temp.idx_to_col(index TEXT)
RETURNS INTEGER AS $idx_to_col$
    SELECT CASE
        WHEN index = '0' THEN 125 | (255 << 8) | (0 << 16)
        WHEN index = '1' THEN 0 | (255 << 8) | (0 << 16)
        WHEN index = '2' THEN 0 | (255 << 8) | (125 << 16)
        WHEN index = '3' THEN 0 | (255 << 8) | (255 << 16)
        WHEN index = '16' THEN 0 | (200 << 8) | (255 << 16)
        WHEN index = '4' THEN 0 | (125 << 8) | (255 << 16)
        WHEN index = '5' THEN 0 | (0 << 8) | (255 << 16)
        WHEN index = '6' THEN 125 | (0 << 8) | (255 << 16)
        WHEN index = '13' THEN 185 | (0 << 8) | (255 << 16)
        WHEN index = '7' THEN 255 | (0 << 8) | (255 << 16)
        WHEN index = '8' THEN 255 | (0 << 8) | (125 << 16)
        WHEN index = '9' THEN 255 | (0 << 8) | (0 << 16)
        WHEN index = '29' THEN 255 | (75 << 8) | (0 << 16)
        WHEN index = '10' THEN 255 | (125 << 8) | (0 << 16)
        WHEN index = '14' THEN 255 | (185 << 8) | (0 << 16)
        WHEN index = '11' THEN 255 | (255 << 8) | (0 << 16)
        WHEN index = '12' THEN 255 | (255 << 8) | (255 << 16)
        WHEN index = '17' THEN 175 | (175 << 8) | (175 << 16)
        WHEN index = '18' THEN 80 | (80 << 8) | (80 << 16)
        WHEN index = '15' THEN 0 | (0 << 8) | (0 << 16)
        WHEN index = '27' THEN 125 | (125 << 8) | (0 << 16)
        WHEN index = '32' THEN 100 | (150 << 8) | (0 << 16)
        WHEN index = '28' THEN 75 | (175 << 8) | (0 << 16)
        WHEN index = '38' THEN 0 | (150 << 8) | (0 << 16)
        WHEN index = '20' THEN 0 | (175 << 8) | (75 << 16)
        WHEN index = '33' THEN 0 | (150 << 8) | (100 << 16)
        WHEN index = '21' THEN 0 | (125 << 8) | (125 << 16)
        WHEN index = '34' THEN 0 | (100 << 8) | (150 << 16)
        WHEN index = '22' THEN 0 | (75 << 8) | (175 << 16)
        WHEN index = '39' THEN 0 | (0 << 8) | (150 << 16)
        WHEN index = '23' THEN 75 | (0 << 8) | (175 << 16)
        WHEN index = '35' THEN 100 | (0 << 8) | (150 << 16)
        WHEN index = '24' THEN 125 | (0 << 8) | (125 << 16)
        WHEN index = '36' THEN 150 | (0 << 8) | (100 << 16)
        WHEN index = '25' THEN 175 | (0 << 8) | (75 << 16)
        WHEN index = '37' THEN 150 | (0 << 8) | (0 << 16)
        WHEN index = '30' THEN 150 | (50 << 8) | (0 << 16)
        WHEN index = '26' THEN 175 | (75 << 8) | (0 << 16)
        WHEN index = '31' THEN 150 | (100 << 8) | (0 << 16)
        WHEN index = '19' THEN 255 | (255 << 8) | (125 << 16)
        WHEN index = '40' THEN 125 | (255 << 8) | (175 << 16)
        WHEN index = '41' THEN 125 | (125 << 8) | (255 << 16)
        ELSE -(index::INTEGER)
    END;
$idx_to_col$ LANGUAGE SQL IMMUTABLE;

ALTER TABLE profile ALTER COLUMN index_10 TYPE INTEGER USING pg_temp.idx_to_col(index_10);
ALTER TABLE profile RENAME COLUMN index_10 TO primary_color;

ALTER TABLE profile ALTER COLUMN index_11 TYPE INTEGER USING pg_temp.idx_to_col(index_11);
ALTER TABLE profile RENAME COLUMN index_11 TO secondary_color;
