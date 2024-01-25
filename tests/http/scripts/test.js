function onEvent(event) {
  if (event.request.method === "GET" && event.request.url === "http://localhost:3000/hello") {
    return {
      response: {
        status: 200,
        headers: {
          "Content-Type": "application/json",
        },
        body: "hello world",
      },
    }
  }
}
