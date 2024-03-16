# test-experimental-headers-error

###### sdl error

```graphql @server
schema @server(headers: {experimental: ["non-experimental", "foo", "bar", "tailcall"]}) {
  query: Query
}

type Query {
  hello: String @const(data: "World!")
}
```
