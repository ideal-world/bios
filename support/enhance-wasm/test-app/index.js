import * as bios from "bios-enhance-wasm";

// pri key : 4a0d660b70a8ee0a46c6ebf8482b158d321e59fab2d15c3fdd89ddaea24144aa
// pub key : 02fbba662032fd384079b7824c07ec8eeaac615187e27ce6a58fcd1597105c1065
bios.init_by_conf({
    "strict_security_mode": false,
    "pub_key": "02fbba662032fd384079b7824c07ec8eeaac615187e27ce6a58fcd1597105c1065",
    "double_auth_exp_sec": 60,
    "apis": [
        {
            "action": "get",
            "uri": "im/ct/need_crypto_req/**",
            "need_crypto_req": true,
            "need_crypto_resp": false,
            "need_double_auth": false
        },
        {
            "action": "get",
            "uri": "im/ct/need_crypto_resp/**",
            "need_crypto_req": false,
            "need_crypto_resp": true,
            "need_double_auth": false
        },
        {
            "action": "get",
            "uri": "im/ct/need_double_auth/**",
            "need_crypto_req": false,
            "need_crypto_resp": false,
            "need_double_auth": true
        },
        {
            "action": "get",
            "uri": "im/cs/**",
            "need_crypto_req": true,
            "need_crypto_resp": true,
            "need_double_auth": false
        }
    ]
});

// request process
console.log("no need encrypt = " + bios.request("post", "im/cs/xxxx", "hello world", {}));
console.log("request & response need encrypt = " + JSON.stringify(bios.request("get", "im/cs/xxxx", "hello world", {})));
console.log("request need encrypt = " + JSON.stringify(bios.request("get", "im/ct/need_crypto_req/xxxx", "hello world", {})));
console.log("response need encrypt = " + JSON.stringify(bios.request("get", "im/ct/need_crypto_resp/xxxx", "hello world", {})));


// ------------------------

const mock_req_body = "中台经过几年“滚雪球”的发展或是资本地运作，已是个“庞然大物”，是到了“减肥”，“减负”的时候。一言以避之：解构中台，让他融合到更大的IT能力共享架构中，把共享交给开放平台，把技术还给技术平台，让中台专注于领域服务及事件 。";

bios.init_by_conf({
    "strict_security_mode": true,
    "pub_key": "02fbba662032fd384079b7824c07ec8eeaac615187e27ce6a58fcd1597105c1065",
    "double_auth_exp_sec": 60,
    "apis": []
});

console.log("strict security mode = " + JSON.stringify(bios.request("post", "im/cs/xxxx", mock_req_body, {})));

const start = new Date().getTime();
Array.from(Array(100)).forEach((x, i) => {
    JSON.stringify(bios.request("post", "im/cs/xxxx", mock_req_body, {}))
});
console.log("average time (ms) :" + (new Date().getTime() - start)/100);


// // double auth
// console.log("need double auth = " + bios.double_auth_need_auth());
// bios.double_auth_set_latest_authed();
// console.log("need double auth = " + bios.double_auth_need_auth());

