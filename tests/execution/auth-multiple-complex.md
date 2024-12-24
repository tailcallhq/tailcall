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
  animals: [Animal!]! @expr(body: [{Dog: {bark: "woof"}}, {Cat: {meow: "meow"}}, {Bird: {tweet: "tweet"}}])
}

union Animal = Dog | Cat | Bird

type Dog {
  bark: String @protected(id: ["a"])
}

type Cat {
  meow: String @protected(id: ["b"])
}

type Bird {
  tweet: String @protected(id: ["c"])
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
        animals {
          ... on Dog {
            __typename
            bark
          }
        }
      }
# TEST: 1 [a,b]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIyOm15cGFzc3dvcmQ= # testuser2:mypassword
  body:
    query: |
      query {
        animals {
          ... on Cat {
            __typename
            meow
          }
        }
      }
# TEST: 2 [b,c]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIzOmFiYzEyMw== # testuser3:abc123
  body:
    query: |
      query {
        animals {
          ... on Bird {
            __typename
            tweet
          }
        }
      }

# TEST: 3 [a,c]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIxOnBhc3N3b3JkMTIz # testuser1:password123
  body:
    query: |
      query {
        animals {
          ... on Bird {
            __typename
            tweet
          }
        }
      }
# TEST: 4 [a,b]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIyOm15cGFzc3dvcmQ= # testuser2:mypassword
  body:
    query: |
      query {
        animals {
          ... on Dog {
            __typename
            bark
          }
        }
      }
# TEST: 5 [b,c]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIzOmFiYzEyMw== # testuser3:abc123
  body:
    query: |
      query {
        animals {
          ... on Cat {
            __typename
            meow
          }
        }
      }

# TEST: 6 [a,c]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIxOnBhc3N3b3JkMTIz # testuser1:password123
  body:
    query: |
      query {
        animals {
          ... on Dog {
            __typename
            bark
          }
          ... on Bird {
            __typename
            tweet
          }
        }
      }
# TEST: 7 [a,b]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIyOm15cGFzc3dvcmQ= # testuser2:mypassword
  body:
    query: |
      query {
        animals {
          ... on Dog {
            __typename
            bark
          }
          ... on Cat {
            __typename
            meow
          }
        }
      }
# TEST: 8 [b,c]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIzOmFiYzEyMw== # testuser3:abc123
  body:
    query: |
      query {
        animals {
          ... on Cat {
            __typename
            meow
          }
          ... on Bird {
            __typename
            tweet
          }
        }
      }

# TEST: 9 [a,c]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIxOnBhc3N3b3JkMTIz # testuser1:password123 this should give autherror because it needs b
  body:
    query: |
      query {
        animals {
          ... on Cat {
            __typename
            meow
          }
        }
      }

# TEST: 10 [a,b]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIyOm15cGFzc3dvcmQ= # testuser2:mypassword this should give autherror
  body:
    query: |
      query {
        animals {
          ... on Bird {
            __typename
            tweet
          }
        }
      }
# TEST: 11 [b,c]
- method: POST
  url: http://localhost:8080/graphql
  headers:
    Authorization: Basic dGVzdHVzZXIzOmFiYzEyMw== # testuser3:abc123 this should give autherror
  body:
    query: |
      query {
        animals {
          ... on Dog {
            __typename
            bark
          }
        }
      }
```
