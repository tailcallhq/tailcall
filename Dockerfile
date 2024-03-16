FROM ubuntu:latest
RUN apt-get update && apt-get install -y curl jq
RUN curl -sSL https://raw.githubusercontent.com/tailcallhq/tailcall/master/install.sh | bash -s
ENV PATH="${PATH}:~/.tailcall/bin"
