function onEvent(event) {
  if (event.message.response) {
    return event
  }
  if (event.message.request.method === "GET" && event.message.request.url === "http://localhost:3000/hello") {
    return {
      message: {
        response: {
          status: 200,
          headers: {
            "Content-Type": "application/json",
          },
          body: "hello world",
        },
      },
    }
  } else if (event.message.request.method === "GET" && event.message.request.url === "http://localhost:3000/hi") {
    return {
      message: {
        request: {
          url: "http://localhost:3000/bye",
          headers: {},
          body: "",
          method: "GET",
        },
      },
    }
  }
}
