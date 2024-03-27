---
check_identity: true
---

# auth

```graphql @server
schema @server(auth: [{id: "basic", basic: {htpasswd: "{{env.BASIC_AUTH}}"}}, {id: "jwt", jwt: {jwks: {data: "{{vars.JWKS}}"}}}], vars: [{key: "JWKS", value: "{\"keys\": []}"}]) @upstream {
  query: Query
}

type Query {
  data: String @const(data: "data") @protected
}
```
