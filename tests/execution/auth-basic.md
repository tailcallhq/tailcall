# Auth with BasicAuth

```yaml @config
server:
  port: 8000
links:
  - id: htpasswd
    src: .htpasswd
    type: Htpasswd
```

```graphql @schema
schema {
  query: Query
  mutation: Mutation
}

type Query {
  scalar: String! @expr(body: "data from public scalar")
  protectedScalar: String! @protected @expr(body: "data from protected scalar")
  nested: Nested! @expr(body: {name: "nested name", protected: "protected nested"})
  protectedType: ProtectedType @expr(body: {name: "protected type name", nested: "protected type nested"})
}

type Mutation {
  protectedType: ProtectedType @http(url: "http://upstream/protected")
}

type Nested {
  name: String!
  protected: String! @protected
}

type ProtectedType @protected {
  name: String!
  nested: String!
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
    url: http://upstream/protected
    headers:
      authorization: Basic dGVzdHVzZXIxOnBhc3N3b3JkMTIz
  response:
    status: 200
    body:
      name: mutation name
      nested: mutation nested
```

```yml @test
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
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: |
      mutation {
        protectedType {
          name
          nested
        }
      }
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIxOnBhc3N3b3JkMTIz
  body:
    query: |
      mutation {
        protectedType {
          name
          nested
        }
      }
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIxOnJhbmRvbV9wYXNzd29yZA=
  body:
    query: |
      query {
        protectedScalar
      }
```
