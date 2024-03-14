local freq_limit_key = KEYS[1];
local method = ARGV[1];
local path = ARGV[2];

local freq_key = string.format('%s', freq_limit_key);
local freq_ts_key = string.format('%s:freq_ts', freq_limit_key);
local current_count = tonumber(redis.call('incr', freq_key));
local current_ts = tonumber(redis.call('time')[1]);
local freq_limit = redis.call('get', freq_limit_key);

if current_count == 1 then
    redis.call('set', freq_ts_key, current_ts);
end

if current_count > tonumber(freq_limit) then
    local last_refresh_time = tonumber(redis.call('get', freq_ts_key));
    if last_refresh_time + 6000 > current_ts then
        return 0;
    end
    redis.call('set', freq_key, '1')
    redis.call('set', freq_ts_key, current_ts);
end
return 1;