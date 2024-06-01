# Grpc datasource

```graphql @config
schema
  @server(port: 8000)
  @upstream(httpCache: 42, baseURL: "http://localhost:50051")
  @link(src: "http://localhost:50051", type: Grpc) {
  query: Query
}

type Query {
  news: NewsData! @grpc(method: "news.NewsService.GetAllNews")
}

type NewsData {
  news: [News]!
}

type News {
  id: Int
  title: String
  body: String
  postImage: String
}
```

```yml @mock
- request:
    method: POST
    url: http://localhost:50051/grpc.reflection.v1alpha.ServerReflection/ServerReflectionInfo
    textBody: \0\0\0\0\x02:\0
  response:
    status: 200
    fileBody: grpc/reflection/news-list-services.bin

- request:
    method: POST
    url: http://localhost:50051/grpc.reflection.v1alpha.ServerReflection/ServerReflectionInfo
    textBody: \0\0\0\0\x12\"\x10news.NewsService
  response:
    status: 200
    fileBody: grpc/reflection/news-service-descriptor.bin

- request:
    method: POST
    url: http://localhost:50051/news.NewsService/GetAllNews
  response:
    status: 200
    textBody: \0\0\0\0t\n#\x08\x01\x12\x06Note 1\x1a\tContent 1\"\x0cPost image 1\n#\x08\x02\x12\x06Note 2\x1a\tContent 2\"\x0cPost image 2
```

```yml @test
- method: POST
  url: http://localhost:8080/graphql
  body:
    query: query { news {news{ id }} }
```
