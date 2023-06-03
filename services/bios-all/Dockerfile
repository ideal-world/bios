FROM mineman/ubuntu:openssl1.1.1t

ENV TZ=Asia/Shanghai

RUN apt-get update && \
    apt-get install -yq tzdata && \
    ln -fs /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone && \
    dpkg-reconfigure -f noninteractive tzdata

# RUN apt update && apt install -y wget && wget http://nz2.archive.ubuntu.com/ubuntu/pool/main/o/openssl/libssl1.1_1.1.1f-1ubuntu2.19_amd64.deb && dpkg -i libssl1.1_1.1.1f-1ubuntu2.19_amd64.deb
RUN apt install -y --reinstall ca-certificates
RUN update-ca-certificates -f
RUN  echo 'ca_certificate=/etc/ssl/certs/ca-certificates.crt'  >> /root/.wgetrc 

WORKDIR /bios

RUN mkdir -p ./config/locale/
COPY ./config/locale/ ./config/locale/
COPY ./bios-serv-all ./bios-serv-all

EXPOSE 8080
EXPOSE 10389

CMD ["./bios-serv-all"]
