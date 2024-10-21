# test-server-base-types

```graphql @config
schema @server(port: 3000){
  query: Query
}

type Query {
  hello: String @expr(body: "world")
}
```

```graphql @config
schema @server(port: 8000) @upstream(proxy: {url: "http://localhost:3000"}) {
  query: Query
}

type Query {
  hello: String @expr(body: "world")
}
```
