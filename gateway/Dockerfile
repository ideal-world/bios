FROM apache/apisix:2.8-alpine

COPY apisix/apisix/plugins/auth-dew.lua /usr/local/apisix/apisix/plugins/auth-dew.lua
COPY apisix/apisix/plugins/auth-dew /usr/local/apisix/apisix/plugins/auth-dew
COPY apisix/conf/config-default.yaml /usr/local/apisix/conf/config-default.yaml

EXPOSE 9080 9443

CMD ["sh", "-c", "/usr/bin/apisix init && /usr/bin/apisix init_etcd && /usr/local/openresty/bin/openresty -p /usr/local/apisix -g 'daemon off;'"]

STOPSIGNAL SIGQUIT