FROM ubuntu

ENV TZ=Asia/Shanghai
RUN ln -snf /usr/share/zoneinfo/$TZ /etc/localtime && echo $TZ > /etc/timezone

WORKDIR /bios

RUN mkdir -p ./config/locale/
COPY ./config/locale/ ./config/locale/
COPY ./bios-serv-all ./bios-serv-all

RUN apt update 
RUN apt install --reinstall ca-certificates
RUN update-ca-certificates -f
RUN  echo 'ca_certificate=/etc/ssl/certs/ca-certificates.crt'  >> /root/.wgetrc 

EXPOSE 8080
EXPOSE 10389

CMD ["./bios-serv-all"]