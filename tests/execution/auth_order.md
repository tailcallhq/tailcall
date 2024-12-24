# auth order

```yaml @config
links:
  - id: htpasswd
    src: .htpasswd
    type: Htpasswd
```

```graphql @schema
schema {
  query: Query
}

type Query {
  data: String @http(url: "http://upstream/data") @protected
}
```

```text @file:.htpasswd
testuser1:$apr1$e3dp9qh2$fFIfHU9bilvVZBl8TxKzL/
testuser2:$2y$10$wJ/mZDURcAOBIrswCAKFsO0Nk7BpHmWl/XuhF7lNm3gBAFH3ofsuu
testuser3:{SHA}Y2fEjdGT1W6nsLqtJbGUVeUp9e4=
```

```yml @mock
- request:
    method: GET
    url: http://upstream/data
  expectedHits: 0
  response:
    status: 500
    body: false
```

```yml @test
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: |
      query {
        data
      }
```
