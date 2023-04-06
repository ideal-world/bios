import * as bios from "bios-enhance-wasm";

// pri key : 4a0d660b70a8ee0a46c6ebf8482b158d321e59fab2d15c3fdd89ddaea24144aa
// pub key : 02fbba662032fd384079b7824c07ec8eeaac615187e27ce6a58fcd1597105c1065
bios.init_by_conf({
    "global_api_mode": false,
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
            "need_double_auth": true
        }
    ]
});

// crypto process
console.log("no need encrypt = " + bios.crypto_encrypt("hello world", "post", "im/cs/xxxx"));
console.log("request & response need encrypt = " + JSON.stringify(bios.crypto_encrypt("hello world", "get", "im/cs/xxxx")));
console.log("request need encrypt = " + JSON.stringify(bios.crypto_encrypt("hello world", "get", "im/ct/need_crypto_req/xxxx")));
console.log("response need encrypt = " + JSON.stringify(bios.crypto_encrypt("hello world", "get", "im/ct/need_crypto_resp/xxxx")));

// double auth
console.log("need double auth = " + bios.double_auth_need_auth());
bios.double_auth_set_latest_authed();
console.log("need double auth = " + bios.double_auth_need_auth());

