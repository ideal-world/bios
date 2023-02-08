FROM apache/apisix:2.15.0-alpine

ENV TZ=Asia/Shanghai
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

COPY apisix/plugins/auth-bios.lua /usr/local/apisix/apisix/plugins/auth-bios.lua
#COPY conf/config-default.yaml /usr/local/apisix/conf/config-default.yaml
COPY conf/config.yaml /usr/local/apisix/conf/config.yaml

VOLUME /usr/local/apisix/conf

EXPOSE 9080 9443

CMD ["sh", "-c", "/usr/bin/apisix init && /usr/bin/apisix init_etcd && /usr/local/openresty/bin/openresty -p /usr/local/apisix -g 'daemon off;'"]

STOPSIGNAL SIGQUIT