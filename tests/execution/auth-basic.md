# Auth with BasicAuth

```graphql @server
schema @server(graphiql: true, port: 8000) @link(id: "htpasswd", src: ".htpasswd", type: Htpasswd) {
  query: Query
}

type Nested {
  name: String! @const(data: "nested name")
  protected: String! @const(data: "protected nested")
}

type ProtectedType {
  name: String! @const(data: "protected type name")
  nested: String! @const(data: "protected type nested")
}

type Query {
  nested: Nested!
  protectedScalar: String! @const(data: "data from protected scalar")
  protectedType: ProtectedType
  scalar: String! @const(data: "data from public scalar")
}
```

```text @file:.htpasswd
testuser1:$apr1$e3dp9qh2$fFIfHU9bilvVZBl8TxKzL/
testuser2:$2y$10$wJ/mZDURcAOBIrswCAKFsO0Nk7BpHmWl/XuhF7lNm3gBAFH3ofsuu
testuser3:{SHA}Y2fEjdGT1W6nsLqtJbGUVeUp9e4=
```

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        scalar
        nested {
          name
        }
      }
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      query {
        protectedScalar
      }
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIxOnJhbmRvbV9wYXNzd29yZA==
  body:
    query: |
      query {
        protectedScalar
      }
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIxOnBhc3N3b3JkMTIz
  body:
    query: |
      query {
        protectedScalar
        nested {
          name
          protected
        }
        protectedType {
          name
          nested
        }
      }
```
