FROM ubuntu:latest
RUN apt-get update && apt-get install -y curl
RUN curl -sSL https://raw.githubusercontent.com/tailcallhq/tailcall/master/install.sh | bash -s -- v0.55.0
ENV PATH="${PATH}:~/.tailcall/bin"
