import {Event, Command} from "./tailcall"

function onRequest(event: Event): Command {
  return {
    response: {
      status: 200,
      headers: {
        "Content-Type": "text/plain",
      },
      body: new TextEncoder().encode(JSON.stringify([{title: "Hello, World!"}])),
    },
  }
}
