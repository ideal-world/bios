FROM ubuntu

ENV TZ=Asia/Shanghai

RUN apt-get update && \
    apt-get install -yq tzdata && \
    ln -fs /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone && \
    dpkg-reconfigure -f noninteractive tzdata

RUN apt install -y --reinstall ca-certificates
RUN update-ca-certificates -f
RUN  echo 'ca_certificate=/etc/ssl/certs/ca-certificates.crt'  >> /root/.wgetrc 

WORKDIR /bios

RUN mkdir -p ./config/locale/
COPY config/locale ./config/locale/
COPY ./bios-spi-serv-all ./bios-spi-serv-all

EXPOSE 8080
EXPOSE 10389

CMD ["./bios-spi-serv-all"]