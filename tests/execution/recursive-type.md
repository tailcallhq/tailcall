```graphql @config
schema {
  query: Query
}

type Query {
  color: Color @http(baseURL: "https://color.com", path: "/")
}

type Color {
  colors: [Color]
  isColorPageExists: Boolean
  isColorsImageAvailable: Boolean
}
```
