// import * as bios from "bios-enhance-wasm"
import { createApp } from 'vue'
import App from './App.vue'
import router from './router'

// console.log('--------Init---------');
// await bios.main('', {
//     "strict_security_mode": false,
//     "pub_key": "02fbba662032fd384079b7824c07ec8eeaac615187e27ce6a58fcd1597105c1065",
//     "double_auth_exp_sec": 60,
//     "apis": [
//         {
//             "action": "get",
//             "uri": "iam/ct/need_crypto_req/**",
//             "need_crypto_req": true,
//             "need_crypto_resp": false,
//             "need_double_auth": false
//         },
//         {
//             "action": "get",
//             "uri": "iam/ct/need_crypto_resp/**",
//             "need_crypto_req": false,
//             "need_crypto_resp": true,
//             "need_double_auth": false
//         },
//         {
//             "action": "get",
//             "uri": "iam/cs/**",
//             "need_crypto_req": true,
//             "need_crypto_resp": true,
//             "need_double_auth": false
//         },
//         {
//             "action": "get",
//             "uri": "iam/ct/need_double_auth/**",
//             "need_crypto_req": false,
//             "need_crypto_resp": false,
//             "need_double_auth": true
//         }],
//     "login_req_method": "put",
//     "login_req_paths": ["iam/cp/login/userpwd"],
//     "logout_req_method": "delete",
//     "logout_req_path": "iam/cp/logout/",
//     "double_auth_req_method": "put",
//     "double_auth_req_path": "iam/cp/login/check",
// });

// console.log('--------Init END---------' + bios.encrypt("test data"));

const app = createApp(App)

app.use(router)

app.mount('#app')
