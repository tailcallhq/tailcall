# Cors

#### server:

```graphql
schema
  @server(
    responseHeaders: [
      {key: "Access-Control-Allow-Origin", value: "*"}
      {key: "Authorization", value: "josaijdfopisjdopfi"}
      {key: "Content-Length", value: "2938453"}
      {key: "Content-Type", value: "application/json"}
      {key: "Access-Control-Allow-Headers", value: "*"}
      {key: "Access-Control-Allow-Methods", value: "POST, GET, OPTIONS"}
    ]
  ) {
  query: Query
}

type Query {
  value: Int @const(data: 1)
}
```

#### assert:

```yml
- method: OPTIONS
  url: http://localhost:8080
  body:
    header:
      Access-Control-Allow-Origin: "*"
      Access-Control-Allow-Headers: "*"
      Access-Control-Allow-Methods: "POST, GET, OPTIONS"
```
