FROM ubuntu

RUN mkdir -p /usr/local/bios & mkdir -p /usr/local/bios/config
COPY ./bios-serv-all /usr/local/bios/bios-serv-all

EXPOSE 8080

CMD ["bios-serv-all"]