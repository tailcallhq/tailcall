function onEvent(event) {
  if (event.response) {
    return {
      response: event.response[0],
    }
  }
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
  } else if (event.request.method === "GET" && event.request.url === "http://localhost:3000/hi") {
    return {
      request: [
        {
          url: "http://localhost:3000/bye",
          headers: {},
          body: "",
          method: "GET",
        },
      ],
    }
  }
}
