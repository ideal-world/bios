FROM ubuntu

# Set time zone
RUN apt-get update && \
  apt-get install -yq tzdata && \
  ln -fs /usr/share/zoneinfo/Asia/Shanghai /etc/localtime && \
  dpkg-reconfigure -f noninteractive tzdata
# When error: `libssl. so.1.1: cannot open shared object file: No such file or directory` appears, please use the following statement to install libssl
# RUN apt update && apt install -y wget && wget http://nz2.archive.ubuntu.com/ubuntu/pool/main/o/openssl/libssl1.1_1.1.1f-1ubuntu2.20_amd64.deb && dpkg -i libssl1.1_1.1.1f-1ubuntu2.20_amd64.deb
RUN apt update && apt install -y --reinstall ca-certificates
RUN update-ca-certificates -f
RUN echo 'ca_certificate=/etc/ssl/certs/ca-certificates.crt'  >> /root/.wgetrc 

RUN sed -i '1i\openssl_conf = default_conf' /usr/lib/ssl/openssl.cnf
RUN echo '[ default_conf ] \n\
  ssl_conf = ssl_sect \n\
  [ssl_sect] \n\
  system_default = system_default_sect \n\
  [system_default_sect] \n\
  MinProtocol = TLSv1 \n\
  CipherString = DEFAULT:@SECLEVEL=1 \n'\
  >>/usr/lib/ssl/openssl.cnf

COPY ./bios-spacegate ./spacegate

CMD ["./spacegate"]