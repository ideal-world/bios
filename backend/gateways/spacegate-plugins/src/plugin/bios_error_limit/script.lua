local current_count = tonumber(redis.call('incr', KEYS[1]));
if current_count == 1 then
    redis.call('set', KEYS[2], ARGV[3]);
end
if current_count > tonumber(ARGV[1]) then
    local last_refresh_time = tonumber(redis.call('get', KEYS[2]));
    if last_refresh_time + tonumber(ARGV[2]) > tonumber(ARGV[3]) then
        if current_count == tonumber(ARGV[1]) + 1 then
            return 1;
        else 
            return 0;
        end
    end
    redis.call('set', KEYS[1], '1')
    redis.call('set', KEYS[2], ARGV[3]);
end

return 2;