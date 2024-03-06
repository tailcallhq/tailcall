# test-scalar-email

#### server:

```graphql
scalar Email

schema @server(port: 8000, graphiql: true, hostname: "localhost") {
  query: Query
}

type Query {
  value(email: Email!): Email! @const(data: "{{args.email}}")
}
```

#### assert:

```yml
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ value(email: "alo@valid.com") }'
```
