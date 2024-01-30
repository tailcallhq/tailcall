# test-server-base-types.graphql

#### server:

```graphql
schema @server(port: 3000) @upstream(baseURL: "http://abc.com") {
  query: Query
}

type Query {
  hello: String @const(data: "world")
}
```

#### server:

```graphql
schema @server(port: 8000) @upstream(proxy: {url: "http://localhost:3000"}) {
  query: Query
}

type Query {
  hello: String @const(data: "world")
}
```

#### merged:

```graphql
schema @server(port: 8000) @upstream(baseURL: "http://abc.com", proxy: {url: "http://localhost:3000"}) {
  query: Query
}

type Query {
  hello: String @const(data: "world")
}
```
