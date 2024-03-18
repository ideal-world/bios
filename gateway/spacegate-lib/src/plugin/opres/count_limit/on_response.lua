-- key
local count_limit_key = KEYS[1];
local count_cumulative_key = string.format('%s:cumulative-count', count_limit_key);

-- status
local status = tonumber(ARGV[1]);

-- if fail, decrease count
if not((status >= 200) or (status < 300)) then
    redis.call('decr', count_cumulative_key);
else