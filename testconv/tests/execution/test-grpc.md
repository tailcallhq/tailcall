# test-grpc

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

type NewsData {
  news: [News]!
}

type Query {
  news: NewsData! @grpc(method: "GetAllNews", protoPath: "tests/graphql/errors/proto/news.proto", service: "news.NewsService")
  newsById(news: NewsInput!): News! @grpc(body: "{{args.news}}", method: "GetNews", protoPath: "tests/graphql/errors/proto/news.proto", service: "news.NewsService")
  newsByIdBatch(news: NewsInput!): News!
    @grpc(body: "{{args.news}}", groupBy: ["news", "id"], method: "GetMultipleNews", protoPath: "tests/graphql/errors/proto/news.proto", service: "news.NewsService")
}
```
