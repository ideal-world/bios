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
            local m_utils = require("apisix.plugins.auth-dew.utils")
            local m_redis = require("apisix.plugins.auth-dew.redis")
            local m_redis1 = require("apisix.plugins.auth-dew.redis")
            m_redis.init("127.0.0.1", 6379, 1, 1000, "123456")
            m_redis1.set("test", "测试1")
            ngx.say(m_redis.get("test"))

            m_redis.hset("test_hash","api://xx/?1","{\"a\":\"xx1\"}")
            m_redis.hset("test_hash","api://xx/?2","{\"a\":\"xx2\"}")
            m_redis.hset("test_hash","api://xx/?3","{\"a\":\"xx3\"}")
            m_redis.hset("test_hash","api://xx/?4","{\"a\":\"xx4\"}")
            m_redis.hset("test_hash","api://xx/?5","{\"a\":\"xx5\"}")
            m_redis.hscan("test_hash","*",2, function(k,v) ngx.say(k..":"..v) end)

            m_redis.hscan("not_exist","*",2, function(k,v) ngx.say(k..":"..v) end)
        }
    }
--- request
GET /t
--- response_body
测试1
api://xx/?1:{"a":"xx1"}
api://xx/?2:{"a":"xx2"}
api://xx/?3:{"a":"xx3"}
api://xx/?4:{"a":"xx4"}
api://xx/?5:{"a":"xx5"}
--- no_error_log
[error]

