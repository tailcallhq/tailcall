wrk.method = "POST"
wrk.body = '{"operationName":null,"variables":{},"query":"{posts{title}}"}'
wrk.headers["Connection"] = "keep-alive"
wrk.headers["User-Agent"] = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36"
wrk.headers["Content-Type"] = "application/json"
