use t::APISIX 'no_plan';

no_long_string();
no_root_location();
no_shuffle();
run_tests;

__DATA__

=== TEST 1: test redis
--- config
    location /t {
        content_by_lua_block {
            local m_utils = require("apisix.plugins.auth-bios.utils")
            local m_redis = require("apisix.plugins.auth-bios.redis")
            local m_redis1 = require("apisix.plugins.auth-bios.redis")
            m_redis.init("127.0.0.1", 6379, 1, 1000, "123456")
            -- reconnect test
            m_redis.init("127.0.0.1", 6379, 1, 1000, "123456")
            -- single instance test
            m_redis1.set("k_test1", "测试1")
            ngx.sleep(1)
            m_redis.set("k_test2", "测试2")
            ngx.sleep(1)
            m_redis.set("k_test3", "测试3")
            ngx.say(m_redis.get("k_test1"))
            -- mock aborted
            m_redis.close()
            ngx.say(m_redis.get("k_test6"))

            ngx.say("-----")

            -- mock aborted
            m_redis.close()
            local list = m_redis.del("k_list")
            m_redis.lpush("k_list", "列表1")
            m_redis.lpush("k_list", "列表2")
            local list = m_redis.lrangeall("k_list")
            ngx.say(list)
            ngx.say(list[1])
            for _, l in pairs(list) do
                ngx.say(l)
            end

            ngx.say("-----")

            m_redis.scan("k_test", 2, function(k,v) ngx.say(k..":"..v) end)
            -- mock aborted
            m_redis.close()
            ngx.say("-----")
            m_redis.scan("k_test", 2, function(k,v) ngx.say(k..":"..v) end)

            ngx.say("-----")

            m_redis.hset("test_hash","api://xx/?1","{\"a\":\"xx1\"}")
            m_redis.hset("test_hash","api://xx/?2","{\"a\":\"xx2\"}")
            m_redis.hset("test_hash","api://xx/?3","{\"a\":\"xx3\"}")
            m_redis.hset("test_hash","api://xx/?4","{\"a\":\"xx4\"}")
            m_redis.hset("test_hash","api://xx/?5","{\"a\":\"xx5\"}")

            ngx.say(m_redis.hget("test_hash","api://xx/?5"))
            ngx.say(m_redis.hget("test_hash","api://xx/?6"))

            ngx.say("-----")

            -- mock aborted
            m_redis.close()
            m_redis.hscan("test_hash","*",2, function(k,v) ngx.say(k..":"..v) end)
            ngx.say("-----")
            m_redis.hscan("test_hash","*",2, function(k,v) ngx.say(k..":"..v) end)
            m_redis.hscan("not_exist","*",2, function(k,v) ngx.say(k..":"..v) end)
        }
    }
--- request
GET /t
--- response_body
测试1
nil
-----
列表2列表1
列表2
列表2
列表1
-----
k_test1:测试1
k_test2:测试2
k_test3:测试3
-----
k_test1:测试1
k_test2:测试2
k_test3:测试3
-----
{"a":"xx5"}
nil
-----
api://xx/?1:{"a":"xx1"}
api://xx/?2:{"a":"xx2"}
api://xx/?3:{"a":"xx3"}
api://xx/?4:{"a":"xx4"}
api://xx/?5:{"a":"xx5"}
-----
api://xx/?1:{"a":"xx1"}
api://xx/?2:{"a":"xx2"}
api://xx/?3:{"a":"xx3"}
api://xx/?4:{"a":"xx4"}
api://xx/?5:{"a":"xx5"}
--- no_error_log
[error]

