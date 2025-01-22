FROM ubuntu:22.04

LABEL org.opencontainers.image.source https://github.com/ideal-world/bios
ENV TZ=Asia/Shanghai

RUN apt-get update && \
    apt-get install -yq tzdata && \
    ln -fs /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone && \
    dpkg-reconfigure -f noninteractive tzdata

RUN apt install -y --reinstall ca-certificates
RUN update-ca-certificates -f
RUN  echo 'ca_certificate=/etc/ssl/certs/ca-certificates.crt'  >> /root/.wgetrc 

RUN sed -i '1i\openssl_conf = default_conf' /usr/lib/ssl/openssl.cnf
RUN echo '[ default_conf ] \n\
ssl_conf = ssl_sect \n\
[ssl_sect] \n\
system_default = system_default_sect \n\
[system_default_sect] \n\
MinProtocol = TLSv1 \n\
CipherString = DEFAULT:@SECLEVEL=1 \n'\
>>/usr/lib/ssl/openssl.cnf

WORKDIR /bios

RUN mkdir -p ./config/locale/
COPY config/locale ./config/locale/
COPY ./bios-event ./bios-event

EXPOSE 8080
EXPOSE 9559

CMD ["./bios-event"]
