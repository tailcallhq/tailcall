# Auth with BasicAuth loaded from const env

#### server:

```graphql
schema @server(port: 8000, graphiql: true, auth: [{id: "basic", basic: {htpasswd: "{{env.HTPASSWD_CONTENT}}"}}]) {
  query: Query
}

type Query {
  scalar: String! @const(data: "data from public scalar")
  protectedScalar: String! @protected @const(data: "data from protected scalar")
  nested: Nested!
  protectedType: ProtectedType
}

type Nested {
  name: String! @const(data: "protected nested name")
  protected: String! @protected @const(data: "protected nested")
}

type ProtectedType @protected {
  name: String! @const(data: "protected type name")
  nested: String! @const(data: "protected type nested")
}
```

#### env:

```yml
HTPASSWD_CONTENT: |
  testuser1:$apr1$e3dp9qh2$fFIfHU9bilvVZBl8TxKzL/
  testuser2:$2y$10$wJ/mZDURcAOBIrswCAKFsO0Nk7BpHmWl/XuhF7lNm3gBAFH3ofsuu
  testuser3:{SHA}Y2fEjdGT1W6nsLqtJbGUVeUp9e4=
```

#### assert:

```yml
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
