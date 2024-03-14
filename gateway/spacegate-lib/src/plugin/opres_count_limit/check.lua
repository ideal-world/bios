local count_limit_key = KEYS[1];

local count_key = string.format('%s:count', count_limit_key);
local count_cumulative_key = string.format('%s:cumulative-count', count_limit_key);
local max_count = tonumber(redis.call('get', count_key));
local current_count = tonumber(redis.call('get', count_cumulative_key));


if max_count < current_count then
    return 0;
else
    redis.call('incr', count_cumulative_key);
    return 1;
end