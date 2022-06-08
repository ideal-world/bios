FROM alpine:3.13 AS etcd-stage

ARG ETCD_VERSION="v3.4.18"

WORKDIR /tmp

RUN wget https://github.com/etcd-io/etcd/releases/download/${ETCD_VERSION}/etcd-${ETCD_VERSION}-linux-amd64.tar.gz \
    && tar -zxvf etcd-${ETCD_VERSION}-linux-amd64.tar.gz \
    && mv etcd-${ETCD_VERSION}-linux-amd64 etcd

FROM apache/apisix:2.14.1-alpine

COPY --from=etcd-stage /tmp/etcd/etcd /usr/bin/etcd
COPY --from=etcd-stage /tmp/etcd/etcdctl /usr/bin/etcdctl

COPY apisix/apisix/plugins/auth-bios.lua /usr/local/apisix/apisix/plugins/auth-bios.lua
COPY apisix/apisix/plugins/auth-bios /usr/local/apisix/apisix/plugins/auth-bios
COPY apisix/conf/config-default.yaml /usr/local/apisix/conf/config-default.yaml

EXPOSE 9080 9443 2379 2380

CMD ["sh", "-c", "(nohup etcd >/tmp/etcd.log 2>&1 &) && sleep 10 && /usr/bin/apisix init && /usr/bin/apisix init_etcd && /usr/local/openresty/bin/openresty -p /usr/local/apisix -g 'daemon off;'"]

STOPSIGNAL SIGQUIT