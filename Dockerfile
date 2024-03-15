FROM ubuntu:latest

RUN curl -sSL https://raw.githubusercontent.com/tailcallhq/tailcall/master/install.sh | bash -s -- v0.55.0
ENV PATH="${PATH}:~/.tailcall/bin"
