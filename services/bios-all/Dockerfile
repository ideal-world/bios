FROM ubuntu

WORKDIR /bios

RUN mkdir ./config
COPY ./bios-serv-all ./bios-serv-all

EXPOSE 8080

CMD ["bios-serv-all"]