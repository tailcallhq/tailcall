# Caching Graphql args

#### server:

```graphql
schema @upstream(baseURL: "http://example.com", batch: {delay: 1, maxSize: 1000}) {
  query: Query
}

type Query @cache(maxAge: 3000) {
  bar(id: Int!, flag: Boolean!, dummy: Dummy): Bar @http(path: "/bar?id={{args.id}}&flag={{args.flag}}")
}

type Dummy {
  list: [Int]
}

type Bar {
  id: Int!
}
```

#### mock:

```yml
- request:
    method: GET
    url: http://example.com/bar?id=1&flag=true
    body: null
  response:
    status: 200
    body:
      id: 1
- request:
    method: GET
    url: http://example.com/bar?id=2&flag=false
    body: null
  response:
    status: 200
    body:
      id: 2
- request:
    method: GET
    url: http://example.com/bar?id=3&flag=false
    body: null
  response:
    status: 200
    body:
      id: 3
```

#### assert:

```yml
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { bar(id: 1, flag: true, dummy: { list: [1] }) { id dir } }"
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { bar(id: 2, flag: false, dummy: { list: [1] }) { id dir } }"
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { bar(id: 3, flag: false, dummy: { list: [1] }) { id dir } }"
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { bar(id: 2, flag: false, dummy: { list: [1] }) { id dir } }"
```
