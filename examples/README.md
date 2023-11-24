# Examples

## Hello World

A simple hello world graphql query.
The tailcall magic occurs with the following directives:
- [@server](https://tailcall.run/docs/guides/operators/#server), which tells tailcall how to configure a server for this file.
- [@upstream](https://tailcall.run/docs/guides/operators/#upstream), which tells tailcall where to make a request to.
- [@http](https://tailcall.run/docs/guides/operators/#http), which tells tailcall a field/node is underpinned by a REST call.

## Extend

Here we explore how to enrich data using tailcall. We make a call to a users service (returning names, emails, usernames), and enriches it with the posts service, which returns posts for a given user.

## Batch

The previous example introduces an N + 1 problem, in that we make 1 request to the users service, and then N requests to the posts service. In this example we flip this around, and make 1 request to the posts service, and then 1 request (with query params) to the users service.