# test-scalar-email

#### server:

```graphql
scalar Email
scalar PhoneNumber

schema @server(port: 8000, graphiql: true, hostname: "localhost") {
  query: Query
}

type Query {
  value(email: Email!): Email! @const(data: "{{args.email}}")
  value_phone(phone: PhoneNumber!): PhoneNumber! @const(data: "{{args.phone}}")
}
```

#### assert:

```yml
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ value(email: "alo@valid.com") }'
- method: POST
  url: http://localhost:8000/graphql
  body:
    query: '{ value_phone(phone: "+1 (614) 1234567") }'
```
