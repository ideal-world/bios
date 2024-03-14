local count_limit_key = KEYS[1];
local status = tonumber(ARGV[1]);
local count_key = string.format('%s:count', count_limit_key);
local count_cumulative_key = string.format('%s:cumulative-count', count_limit_key);
local max_count = tonumber(redis.call('get', count_key));
local current_count = tonumber(redis.call('get', count_cumulative_key));

-- if fail, decrease count
if not((status >= 200) or (status < 300)) then
    redis.call('decr', count_cumulative_key);
else