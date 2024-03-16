# Runner stage
FROM alpine:latest AS runner

RUN apk update && apk add --no-cache curl wget jq bash

# Copy the graphql file
COPY examples/jsonplaceholder.graphql ./

# Install latest version
RUN wget https://raw.githubusercontent.com/tailcallhq/tailcall/master/install.sh
RUN chmod +x install.sh
RUN bash install.sh

ENV TAILCALL_LOG_LEVEL=error
CMD ["/root/.tailcall/bin/tailcall", "start", "jsonplaceholder.graphql"]
