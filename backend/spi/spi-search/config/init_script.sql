CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE EXTENSION IF NOT EXISTS zhparser;

DO
$$BEGIN
    CREATE TEXT SEARCH CONFIGURATION public.chinese_zh (PARSER = zhparser);
    ALTER TEXT SEARCH CONFIGURATION public.chinese_zh ADD MAPPING FOR n,v,a,i,e,l,d,f,j,m,o,p,q,r,u,w,x,y,z WITH simple;
EXCEPTION
   WHEN unique_violation THEN
      NULL;  -- ignore error
END;$$;

-- test
-- SELECT plainto_tsquery('public.chinese_zh', '保障房资金压力');

-- list all tokens
-- select ts_token_type('zhparser');