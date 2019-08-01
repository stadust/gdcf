CREATE FUNCTION pg_temp.col_to_idx(color INTEGER)
RETURNS TEXT AS $col_to_idx$
    SELECT CASE
        WHEN color = 125 | (255 << 8) | (0 << 16) then '0'
        WHEN color = 0 | (255 << 8) | (0 << 16) then '1'
        WHEN color = 0 | (255 << 8) | (125 << 16) then '2'
        WHEN color = 0 | (255 << 8) | (255 << 16) then '3'
        WHEN color = 0 | (200 << 8) | (255 << 16) then '16'
        WHEN color = 0 | (125 << 8) | (255 << 16) then '4'
        WHEN color = 0 | (0 << 8) | (255 << 16) then '5'
        WHEN color = 125 | (0 << 8) | (255 << 16) then '6'
        WHEN color = 185 | (0 << 8) | (255 << 16) then '13'
        WHEN color = 255 | (0 << 8) | (255 << 16) then '7'
        WHEN color = 255 | (0 << 8) | (125 << 16) then '8'
        WHEN color = 255 | (0 << 8) | (0 << 16) then '9'
        WHEN color = 255 | (75 << 8) | (0 << 16) then '29'
        WHEN color = 255 | (125 << 8) | (0 << 16) then '10'
        WHEN color = 255 | (185 << 8) | (0 << 16) then '14'
        WHEN color = 255 | (255 << 8) | (0 << 16) then '11'
        WHEN color = 255 | (255 << 8) | (255 << 16) then '12'
        WHEN color = 175 | (175 << 8) | (175 << 16) then '17'
        WHEN color = 80 | (80 << 8) | (80 << 16) then '18'
        WHEN color = 0 | (0 << 8) | (0 << 16) then '15'
        WHEN color = 125 | (125 << 8) | (0 << 16) then '27'
        WHEN color = 100 | (150 << 8) | (0 << 16) then '32'
        WHEN color = 75 | (175 << 8) | (0 << 16) then '28'
        WHEN color = 0 | (150 << 8) | (0 << 16) then '38'
        WHEN color = 0 | (175 << 8) | (75 << 16) then '20'
        WHEN color = 0 | (150 << 8) | (100 << 16) then '33'
        WHEN color = 0 | (125 << 8) | (125 << 16) then '21'
        WHEN color = 0 | (100 << 8) | (150 << 16) then '34'
        WHEN color = 0 | (75 << 8) | (175 << 16) then '22'
        WHEN color = 0 | (0 << 8) | (150 << 16) then '39'
        WHEN color = 75 | (0 << 8) | (175 << 16) then '23'
        WHEN color = 100 | (0 << 8) | (150 << 16) then '35'
        WHEN color = 125 | (0 << 8) | (125 << 16) then '24'
        WHEN color = 150 | (0 << 8) | (100 << 16) then '36'
        WHEN color = 175 | (0 << 8) | (75 << 16) then '25'
        WHEN color = 150 | (0 << 8) | (0 << 16) then '37'
        WHEN color = 150 | (50 << 8) | (0 << 16) then '30'
        WHEN color = 175 | (75 << 8) | (0 << 16) then '26'
        WHEN color = 150 | (100 << 8) | (0 << 16) then '31'
        WHEN color = 255 | (255 << 8) | (125 << 16) then '19'
        WHEN color = 125 | (255 << 8) | (175 << 16) then '40'
        WHEN color = 125 | (125 << 8) | (255 << 16) then '41'
        ELSE (-color)::TEXT
    END;
$col_to_idx$ LANGUAGE SQL IMMUTABLE;

ALTER TABLE profile RENAME COLUMN primary_color TO index_10;
ALTER TABLE profile ALTER COLUMN index_10 TYPE TEXT USING pg_temp.col_to_idx(index_10);

ALTER TABLE profile RENAME COLUMN secondary_color TO index_11;
ALTER TABLE profile ALTER COLUMN index_11 TYPE TEXT USING pg_temp.col_to_idx(index_11);
