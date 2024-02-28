interface HttpRequest {
  url: string
  method: string
  headers: {[key: string]: string}
}

interface CommandRequest extends HttpRequest {
  body?: string
}

interface HttpResponse {
  status: number
  headers: {[key: string]: string}
  body?: string
}

type Command = {request: CommandRequest} | {response: HttpResponse}

function onRequest(request: HttpRequest): Command {
  return {
    response: {
      status: 200,
      headers: {
        "Content-Type": "text/plain",
      },
      body: JSON.stringify([{title: "Hello, World!"}]),
    },
  }
}
