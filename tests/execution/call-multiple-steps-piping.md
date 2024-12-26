# Call multiple steps piping

```graphql @schema
schema {
  query: Query
}

type Query {
  a_input(input: JSON): JSON @expr(body: {input: "{{.args.input.a}}"})
  b_input(input: JSON): JSON @expr(body: {input: "{{.args.input.b}}"})
  a(input: JSON): JSON @expr(body: "{{.args.input.a}}")
  b(input: JSON): JSON @expr(body: "{{.args.input.b}}")
  c(input: JSON): JSON @expr(body: "{{.args.input.c}}")
  wrap_args: JSON @expr(body: {input: "{{.args}}"})
  wrap_input(input: JSON): JSON @expr(body: {input: "{{.args.input}}"})

  abc_input(input: JSON): JSON
    @call(
      steps: [
        {query: "wrap_input", args: {input: "{{.args.input}}"}}
        {query: "a_input"}
        {query: "wrap_input"}
        {query: "b_input"}
        {query: "wrap_input"}
        {query: "c"}
      ]
    )
  abc(input: JSON): JSON
    @call(
      steps: [
        {query: "a", args: {input: "{{.args.input}}"}}
        {query: "wrap_args"}
        {query: "b"}
        {query: "wrap_args"}
        {query: "c"}
      ]
    )
}
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { abc_input(input: {a: {b: {c: 3}}})}"
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { abc(input: {a: {b: {c: 3}}}) }"
```
