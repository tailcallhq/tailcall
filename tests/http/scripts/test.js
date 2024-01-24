function onEvent(event) {
  if (event.request.url === "/test") {
    let obj = {
      response: {
        status: 200,
        headers: {
          "Content-Type": "text/plain",
        },
        body: "Hello World!",
      },
    }
    return obj
  } else if (event.request.url === "/test2") {
    let obj = {
      request: [
        {
          method: "GET",
          url: "/test",
        },
        {
          method: "GET",
          url: "/test",
        },
      ],
    }
    return obj
  }
}
