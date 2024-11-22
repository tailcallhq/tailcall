# auth multiple

```graphql @config
schema
  @server
  @upstream
  @link(id: "a", src: ".htpasswd_a", type: Htpasswd)
  @link(id: "b", src: ".htpasswd_b", type: Htpasswd)
  @link(id: "c", src: ".htpasswd_c", type: Htpasswd) {
  query: Query
}

type Query {
  default: String @expr(body: "data") @protected
  a_and_b: String @expr(body: "data") @protected(providers: ["a", "b"])
  b_and_c: String @expr(body: "data") @protected(providers: ["b", "c"])
  c_and_a: String @expr(body: "data") @protected(providers: ["c", "a"])
  a_or_b: String @expr(body: "data") @protected(providers: ["a"]) @protected(providers: ["b"])
  b_or_c: String @expr(body: "data") @protected(providers: ["b"]) @protected(providers: ["c"])
  c_or_a: String @expr(body: "data") @protected(providers: ["c"]) @protected(providers: ["a"])
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
    Authorization: Basic dGVzdHVzZXIxOnBhc3N3b3JkMTIz # testuser1:password123
  body:
    query: |
      query {
        a_and_b
      }
# TEST: 1 [a,b]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIyOm15cGFzc3dvcmQ= # testuser2:mypassword
  body:
    query: |
      query {
        a_and_b
      }
# TEST: 2 [b,c]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIzOmFiYzEyMw== # testuser3:abc123
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
        a_or_b
      }
# TEST: 10 [a,b]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIyOm15cGFzc3dvcmQ=
  body:
    query: |
      query {
        a_or_b
      }
# TEST: 11 [b,c]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIzOmFiYzEyMw==
  body:
    query: |
      query {
        a_or_b
      }
# TEST: 12 [a,c]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIxOnBhc3N3b3JkMTIz
  body:
    query: |
      query {
        b_or_c
      }
# TEST: 13 [a,b]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIyOm15cGFzc3dvcmQ=
  body:
    query: |
      query {
        b_or_c
      }
# TEST: 14 [b,c]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIzOmFiYzEyMw==
  body:
    query: |
      query {
        b_or_c
      }
# TEST: 15 [a,c]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIxOnBhc3N3b3JkMTIz
  body:
    query: |
      query {
        c_or_a
      }
# TEST: 16 [a,b]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIyOm15cGFzc3dvcmQ=
  body:
    query: |
      query {
        c_or_a
      }
# TEST: 17 [b,c]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIzOmFiYzEyMw==
  body:
    query: |
      query {
        c_or_a
      }

# TEST: 18 [a,c]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIxOnBhc3N3b3JkMTIz
  body:
    query: |
      query {
        default
      }
# TEST: 19 [a,b]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIyOm15cGFzc3dvcmQ=
  body:
    query: |
      query {
        default
      }
# TEST: 20 [b,c]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIzOmFiYzEyMw==
  body:
    query: |
      query {
        default
      }

# TEST: 21 [a,c]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIxOnBhc3N3b3JkMTIz # testuser1:password123
  body:
    query: |
      query {
        c_and_a
        a_or_b
        b_or_c
        c_or_a
        default
      }
# TEST: 22 [a,b]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIyOm15cGFzc3dvcmQ= # testuser2:mypassword
  body:
    query: |
      query {
        a_and_b
        a_or_b
        b_or_c
        c_or_a
        default
      }
# TEST: 23 [b,c]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIzOmFiYzEyMw== # testuser3:abc123
  body:
    query: |
      query {
        b_and_c
        a_or_b
        b_or_c
        c_or_a
        default
      }
```