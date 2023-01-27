## example project/示例
run by docker-compose

### step 1:
build apisix iamges
```shell
 docker build -f ../Apisix_Dockerfile -t test/apisix:v1 ../
```
### step 2:
```shell
docker-compose up -d
```
### step 3:
run ./auth rust project(MockOPA service)

### step 4:
open dashboard server,crete route.\
set opa plugins conf:
`{"host":"http://172.23.159.226:8080","policy":"auth_apisix"}` \
or use this command:
```shell
curl 'http://localhost:9180/apisix/admin/routes/1' \
  -X 'PUT' \
  -H "X-API-KEY: edd1c9f034335f136f87ad84b6acecs1" \
  --data-raw '{
  "uri":"/*","name":"1","methods":["GET","POST","PUT","DELETE","PATCH","HEAD","OPTIONS","CONNECT","TRACE","PURGE"],
  "plugins":{"opa":{"disable":false,"host":"[your host ip:8080]","policy":"auth_apisix"}},
  "upstream":{"nodes":[{"host":"http://web1","port":9081,"weight":1}],
  "timeout":{"connect":6,"send":6,"read":6},"type":"roundrobin","scheme":"http","pass_host":"pass","keepalive_pool":{"idle_timeout":60,"requests":1000,"size":320}},"status":1
  }' \
  --compressed
```

