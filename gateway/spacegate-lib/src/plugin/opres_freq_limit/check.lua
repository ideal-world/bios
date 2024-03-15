local freq_limit_key = KEYS[1];

local freq_key = string.format('%s:freq', freq_limit_key);
local freq_ts_key = string.format('%s:freq_ts', freq_limit_key);
if not(redis.call('exists', freq_key)) then
    redis.call('set', freq_key, 0);
end
local current_count = tonumber(redis.call('incr', freq_key));
local current_ts = tonumber(redis.call('time')[1]);
local freq_limit = tonumber(redis.call('get', freq_limit_key));

if current_count == 1 then
    redis.call('set', freq_ts_key, current_ts);
end

if current_count > freq_limit then
    local last_refresh_time = tonumber(redis.call('get', freq_ts_key));
    redis.call('set', 'debug:time', string.format('%d,%d', last_refresh_time, current_ts))
    if last_refresh_time + 60 > current_ts then
        return 0;
    else
        redis.call('set', freq_key, '1')
        redis.call('set', freq_ts_key, current_ts);
    end
end
return 1;