FROM ubuntu

RUN apt-get update && \
    apt-get install -yq tzdata && \
    ln -fs /usr/share/zoneinfo/America/New_York /etc/localtime && \
    dpkg-reconfigure -f noninteractive tzdata

ENV TZ=Asia/Shanghai
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

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