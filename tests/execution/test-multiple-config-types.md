# Multiple Configs

```yaml @config
links:
  - id: types
    type: Config
    src: types.graphql
```

```graphql @schema
schema {
  query: Query
}

type Query {
  bar(input: Input): Output @expr(body: {id: "{{.args.input.id}}", name: "{{.args.input.name}}"})
}
```

```graphql @file:types.graphql
input Input {
  id: Int
  name: String
}

type Output {
  id: Int
  name: String
}
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: 'query { bar(input: {id: 1, name: "name"}) { name id } }'
```
