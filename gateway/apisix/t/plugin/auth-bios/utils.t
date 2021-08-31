use t::APISIX 'no_plan';

no_long_string();
no_root_location();
no_shuffle();
run_tests;

__DATA__

=== TEST 1: test utils
--- config
    location /t {
        content_by_lua_block {
            local m_utils = require("apisix.plugins.auth-bios.utils")
            ngx.say(m_utils.start_with("/index/path/", ""))
            ngx.say(m_utils.start_with("/index/path/", "/index"))
            ngx.say(m_utils.start_with("/index/path/", "/index1"))
            ngx.say(m_utils.end_with("/index/path/", ""))
            ngx.say(m_utils.end_with("/index/path/", "path/"))
            ngx.say(m_utils.end_with("/index/path/", "path/1"))

            local result = m_utils.split("1343,334444,", ",")
            ngx.say(result[1])
            ngx.say(result[2])
            ngx.say(result[3])

            local result = m_utils.split("123", "")
            ngx.say(result[1])
            ngx.say(result[2])
            ngx.say(result[3])

             ngx.say(m_utils.contain("/index/path/", "path"))
             ngx.say(m_utils.contain("/index/path/", "/path"))
             ngx.say(m_utils.contain("/index/path/", "/path1"))

             ngx.say(m_utils.table_length({1,2,3}))
             ngx.say(m_utils.table_length(m_utils.split("1343,334444,", ",")))
        }
    }
--- request
GET /t
--- response_body
true
true
false
true
true
false
1343
334444
nil
1
2
3
true
true
false
3
2
--- no_error_log
[error]

