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

#### assert:
```yml
mock:
- request:
    method: GET
    url: http://example.com/bar?id=1&flag=true
    headers: {}
    body: null
  response:
    status: 200
    headers: {}
    body:
      id: 1
- request:
    method: GET
    url: http://example.com/bar?id=2&flag=false
    headers: {}
    body: null
  response:
    status: 200
    headers: {}
    body:
      id: 2
- request:
    method: GET
    url: http://example.com/bar?id=3&flag=false
    headers: {}
    body: null
  response:
    status: 200
    headers: {}
    body:
      id: 3
assert:
- request:
    method: POST
    url: http://localhost:8080/graphql
    headers: {}
    body:
      query: 'query { bar(id: 1, flag: true, dummy: { list: [1] }) { id dir } }'
- request:
    method: POST
    url: http://localhost:8080/graphql
    headers: {}
    body:
      query: 'query { bar(id: 2, flag: false, dummy: { list: [1] }) { id dir } }'
- request:
    method: POST
    url: http://localhost:8080/graphql
    headers: {}
    body:
      query: 'query { bar(id: 3, flag: false, dummy: { list: [1] }) { id dir } }'
- request:
    method: POST
    url: http://localhost:8080/graphql
    headers: {}
    body:
      query: 'query { bar(id: 2, flag: false, dummy: { list: [1] }) { id dir } }'
env: {}
```
