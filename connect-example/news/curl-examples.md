# Connect RPC Curl Examples

## Using Connect Protocol

### Get All News
```bash
curl \
  --header "Content-Type: application/json" \
  --request POST \
  --data '{}' \
  http://localhost:8080/news.NewsService/GetAllNews
```

### Get Single News
```bash
curl \
  --header "Content-Type: application/json" \
  --request POST \
  --data '{"id": 1}' \
  http://localhost:8080/news.NewsService/GetNews
```

### Get Multiple News
```bash
curl \
  --header "Content-Type: application/json" \
  --request POST \
  --data '{"ids": [{"id": 1}, {"id": 2}]}' \
  http://localhost:8080/news.NewsService/GetMultipleNews
```

### Add News
```bash
curl \
  --header "Content-Type: application/json" \
  --request POST \
  --data '{
    "title": "New Article",
    "body": "This is a new article",
    "postImage": "https://via.placeholder.com/150",
    "status": "PUBLISHED"
  }' \
  http://localhost:8080/news.NewsService/AddNews
```

### Edit News
```bash
curl \
  --header "Content-Type: application/json" \
  --request POST \
  --data '{
    "id": 1,
    "title": "Updated Article",
    "body": "This article has been updated",
    "postImage": "https://via.placeholder.com/150",
    "status": "PUBLISHED"
  }' \
  http://localhost:8080/news.NewsService/EditNews
```

### Delete News
```bash
curl \
  --header "Content-Type: application/json" \
  --request POST \
  --data '{"id": 1}' \
  http://localhost:8080/news.NewsService/DeleteNews
```