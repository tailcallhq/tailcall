# Call multiple steps piping

```graphql @server
schema {
  query: Query
}

type Query {
  a_input(input: JSON): JSON @const(data: {input: "{{args.input.a}}"})
  b_input(input: JSON): JSON @const(data: {input: "{{args.input.b}}"})
  a(input: JSON): JSON @const(data: "{{args.input.a}}")
  b(input: JSON): JSON @const(data: "{{args.input.b}}")
  c(input: JSON): JSON @const(data: "{{args.input.c}}")
  wrap_args: JSON @const(data: {input: "{{args}}"})
  wrap_input(input: JSON): JSON @const(data: {input: "{{args.input}}"})

  abc_input(input: JSON): JSON
    @call(
      steps: [
        {query: "wrap_input", args: {input: "{{args.input}}"}}
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
        {query: "wrap_input", args: {input: "{{args.input}}"}}
        {query: "a"}
        {query: "wrap_args"}
        {query: "b"}
        {query: "wrap_args"}
        {query: "c"}
      ]
    )
}
```

```yml @assert
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { abc_input(input: {a: {b: {c: 3}}})}"
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: "query { abc(input: {a: {b: {c: 3}}}) }"
```
