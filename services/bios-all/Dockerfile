FROM ubuntu

WORKDIR /bios

RUN mkdir ./config
COPY ./bios-serv-all ./bios-serv-all

EXPOSE 8080
EXPOSE 10389

CMD ["./bios-serv-all"]