CREATE OR REPLACE FUNCTION GRAPH_SEARCH(
  IN I_SCHEMA CHARACTER varying,
  IN I_ROOT_KEY CHARACTER varying,
  IN I_ROOT_VERSION CHARACTER varying,
  IN I_DEPTH int DEFAULT 99999,
  IN I_LIMIT int8 DEFAULT 2000000000,
 OUT O_TAG CHARACTER varying,
 OUT O_FROM_KEY CHARACTER varying,
 OUT O_FROM_VERSION CHARACTER varying,
 OUT O_TO_KEY CHARACTER varying,
 OUT O_TO_VERSION CHARACTER varying,
 OUT O_TAGS CHARACTER varying[],
 OUT O_DEPTH int,
 OUT O_PATHS CHARACTER varying[],
 OUT O_REVERSE BOOL) RETURNS
SETOF RECORD AS $$
declare
  sql text;
begin
sql := format($_$
WITH RECURSIVE search_graph(
  tag, from_key, from_version, to_key, to_version, tags, depth, paths, reverse
) AS (
        select tag, from_key, from_version, to_key, to_version, tags, depth, paths, reverse from (
        SELECT
          g.tag,
          g.from_key,
          g.from_version,
          g.to_key,
          g.to_version,
	      ARRAY[g.tag] as tags,
          1 as depth,
          ARRAY[(g.from_key || g.from_version)::varchar, (g.to_key || g.to_version)::varchar] as paths,
          g.reverse
			FROM %s.starsys_graph AS g
        WHERE
          from_key = '%s' AND from_version = '%s'
          limit %s
        ) t
      UNION ALL
        select tag, from_key, from_version, to_key, to_version, tags, depth, paths, reverse from (
        SELECT
          DISTINCT ON (g.from_key, g.from_version, g.to_key, g.to_version, g.tag)
          g.tag,
          g.from_key,
          g.from_version,
          g.to_key,
          g.to_version,
	      sg.tags || ARRAY[g.tag] as tags,
          sg.depth + 1 as depth,
          sg.paths || ARRAY[g.to_key || g.to_version] as paths,
          g.reverse
			FROM %s.starsys_graph AS g, search_graph AS sg
        WHERE
          g.from_key = sg.to_key AND g.from_version = sg.to_version
          AND g.to_key || g.to_version <> ALL(sg.paths)
          AND sg.depth <= %s AND sg.reverse = g.reverse
          limit %s
        ) t
)
SELECT tag as o_tag, from_key as o_from_key, from_version as o_from_version, to_key as o_to_key, to_version as o_to_version, tags as o_tags, depth as o_depth, paths as o_paths, reverse as o_reverse
FROM search_graph;
$_$, i_schema, i_root_key, i_root_version, i_limit, i_schema, i_depth, i_limit
);
return query execute sql;
end;
$$ LANGUAGE PLPGSQL STRICT;