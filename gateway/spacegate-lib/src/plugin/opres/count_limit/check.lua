-- key
local count_limit_key = KEYS[1];
local count_key = string.format('%s', count_limit_key);
local count_cumulative_key = string.format('%s:cumulative-count', count_limit_key);

-- count
local max_count = tonumber(redis.call('get', count_key));
local current_count = tonumber(redis.call('get', count_cumulative_key));
if current_count == nil then
    redis.call('set', count_cumulative_key, '1')
    current_count = 1
end

if max_count < current_count then
    return 0;
else
    redis.call('incr', count_cumulative_key);
    return 1;
end