# auth multiple

```yaml @config
links:
  - id: a
    src: .htpasswd_a
    type: Htpasswd
  - id: b
    src: .htpasswd_b
    type: Htpasswd
  - id: c
    src: .htpasswd_c
    type: Htpasswd
```

```graphql @schema
schema {
  query: Query
}

type Query {
  default: String @expr(body: "data") @protected
  a_and_b: String @expr(body: "data") @protected(id: ["a", "b"])
  b_and_c: String @expr(body: "data") @protected(id: ["b", "c"])
  c_and_a: String @expr(body: "data") @protected(id: ["c", "a"])
}
```

```text @file:.htpasswd_a
testuser1:$apr1$e3dp9qh2$fFIfHU9bilvVZBl8TxKzL/
testuser2:$2y$10$wJ/mZDURcAOBIrswCAKFsO0Nk7BpHmWl/XuhF7lNm3gBAFH3ofsuu
```

```text @file:.htpasswd_b
testuser2:$2y$10$wJ/mZDURcAOBIrswCAKFsO0Nk7BpHmWl/XuhF7lNm3gBAFH3ofsuu
testuser3:{SHA}Y2fEjdGT1W6nsLqtJbGUVeUp9e4=
```

```text @file:.htpasswd_c
testuser1:$apr1$e3dp9qh2$fFIfHU9bilvVZBl8TxKzL/
testuser3:{SHA}Y2fEjdGT1W6nsLqtJbGUVeUp9e4=
```

```yml @test
# TEST: 0 [a,c]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIxOnBhc3N3b3JkMTIz # testuser1:password123 this should give error
  body:
    query: |
      query {
        a_and_b
      }
# TEST: 1 [a,b]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIyOm15cGFzc3dvcmQ= # testuser2:mypassword this should give data
  body:
    query: |
      query {
        a_and_b
      }
# TEST: 2 [b,c]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIzOmFiYzEyMw== # testuser3:abc123 this should give data
  body:
    query: |
      query {
        a_and_b
      }
# TEST: 3 [a,c]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIxOnBhc3N3b3JkMTIz
  body:
    query: |
      query {
        b_and_c
      }
# TEST: 4 [a,b]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIyOm15cGFzc3dvcmQ=
  body:
    query: |
      query {
        b_and_c
      }
# TEST: 5 [b,c]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIzOmFiYzEyMw==
  body:
    query: |
      query {
        b_and_c
      }
# TEST: 6 [a,c]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIxOnBhc3N3b3JkMTIz
  body:
    query: |
      query {
        c_and_a
      }
# TEST: 7 [a,b]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIyOm15cGFzc3dvcmQ=
  body:
    query: |
      query {
        c_and_a
      }
# TEST: 8 [b,c]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIzOmFiYzEyMw==
  body:
    query: |
      query {
        c_and_a
      }

# TEST: 9 [a,c]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIxOnBhc3N3b3JkMTIz
  body:
    query: |
      query {
        default
      }
# TEST: 10 [a,b]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIyOm15cGFzc3dvcmQ=
  body:
    query: |
      query {
        default
      }
# TEST: 11 [b,c]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIzOmFiYzEyMw==
  body:
    query: |
      query {
        default
      }

# TEST: 12 [a,c]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIxOnBhc3N3b3JkMTIz # testuser1:password123
  body:
    query: |
      query {
        c_and_a
        default
      }
# TEST: 13 [a,b]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIyOm15cGFzc3dvcmQ= # testuser2:mypassword
  body:
    query: |
      query {
        a_and_b
        default
      }
# TEST: 14 [b,c]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIzOmFiYzEyMw== # testuser3:abc123
  body:
    query: |
      query {
        b_and_c
        default
      }
```
