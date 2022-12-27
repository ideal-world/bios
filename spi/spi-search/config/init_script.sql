CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE EXTENSION IF NOT EXISTS zhparser;

DO
$$BEGIN
    CREATE TEXT SEARCH CONFIGURATION chinese_zh (PARSER = zhparser);
    ALTER TEXT SEARCH CONFIGURATION chinese_zh ADD MAPPING FOR n,v,a,i,e,l,t WITH simple;
EXCEPTION
   WHEN unique_violation THEN
      NULL;  -- ignore error
END;$$;