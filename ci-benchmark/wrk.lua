wrk.method = "POST"
wrk.body = '{"operationName":null,"variables":{},{"query": "{posts {id,userId, title, user{id, name, email}}}"}}'
wrk.headers["Connection"] = "keep-alive"
wrk.headers["Content-Type"] = "application/json"
