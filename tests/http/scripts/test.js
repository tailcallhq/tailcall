function onEvent(event) {
  if (event.request.method === "GET" && event.request.url === "http://jsonplaceholder.typicode.com/posts") {
    return {
      response: {
        status: 200,
        headers: {
          "Content-Type": "application/json",
        },
        body: [
          {
            id: 1,
            userId: 1,
            title: "sunt aut facere repellat provident occaecati excepturi optio reprehenderit",
            body: "hello world",
          },
          {
            id: 2,
            userId: 2,
            title: "qui est esse",
            body: "hi",
          },
        ],
      },
    }
  }
}
