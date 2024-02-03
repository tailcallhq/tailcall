# test-expr-success

###### check identity

#### server:

```graphql
schema @server(port: 8000) @upstream(baseURL: "http://localhost:50051", batch: {delay: 10, headers: [], maxSize: 1000}, httpCache: true) {
  query: Query
}

input NewsInput {
  body: String
  id: Int
  postImage: String
  title: String
}

type News {
  body: String
  id: Int
  postImage: String
  title: String
}

type Post {
  content: String @expr(body: {graphQL: {args: [{key: "id", value: "{{value.id}}"}], name: "postContent"}})
  id: Int!
}

type Query {
  cond: Post @expr(body: {if: {cond: {const: true}, else: {http: {path: "/posts/1"}}, then: {http: {path: "/posts/2"}}}})
  greeting: String @expr(body: {const: "hello from server"})
  news(news: NewsInput!): News! @expr(body: {grpc: {body: "{{args.news}}", groupBy: ["news", "id"], method: "GetMultipleNews", protoPath: "src/grpc/tests/news.proto", service: "news.NewsService"}})
  post(id: Int!): Post @expr(body: {http: {baseURL: "http://jsonplacheholder.typicode.com", path: "/posts/{{args.id}}"}})
}
```
