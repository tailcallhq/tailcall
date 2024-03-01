# Cors

#### server:

```graphql
schema
  @server(
    responseHeaders: [
      {key: "Access-Control-Allow-Origin", value: "*"}
      {key: "Access-Control-Allow-Headers", value: "*"}
      {key: "Access-Control-Allow-Methods", value: "POST, GET, OPTIONS"}
    ]
  ) {
  query: Query
}

type Query {
  value: Int! @http(path: "/", baseURL: "http://api.com")
}
```

#### mock:

```yml
- request:
    method: POST
    url: http://api.com
    body: null
  response:
    status: 200
    body:
      value: 1
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
