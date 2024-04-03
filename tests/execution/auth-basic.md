# Auth with BasicAuth

```graphql @server
schema @server(port: 8000, graphiql: true) @link(id: "htpasswd", type: Htpasswd, src: ".htpasswd") {
  query: Query
}

type Query {
  scalar: String! @expr(body: "data from public scalar")
  protectedScalar: String! @protected @expr(body: "data from protected scalar")
  nested: Nested! @expr(body: {name: "nested name", protected: "protected nested"})
  protectedType: ProtectedType
}

type Nested {
  name: String!
  protected: String! @protected
}

type ProtectedType @protected {
  name: String! @expr(body: "protected type name")
  nested: String! @expr(body: "protected type nested")
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
