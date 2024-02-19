interface JsRequest {
  url: string
  method: string
  headers: {[key: string]: string} // Represents a map of string to string
  body?: ArrayBuffer | Uint8Array // Optional, use ArrayBuffer or Uint8Array for binary data
}

interface JsResponse {
  status: number
  headers: {[key: string]: string} // Represents a map of string to string
  body?: ArrayBuffer | Uint8Array // Optional, similar handling for binary data
}
interface Continue<T> {
  message: T
  id: number // TypeScript uses 'number' for both integers and floating-point values
}

type Message = {request: JsRequest} | {response: Continue<JsResponse>}
type Command = {request: Continue<JsRequest>} | {response: JsResponse}

declare const channel: {
  addListener(listener: (event: Message) => void): void
  dispatch(message: Command): void
}

channel.addListener((event) => {
  console.log(event)
  if ("request" in event) {
    const request = event.request
    handle(request).then((response) => {
      console.log("Sending response", response)
      channel.dispatch({response})
    })
  }
})

async function handle(request: JsRequest): Promise<JsResponse> {
  return {
    status: 200,
    headers: {
      "Content-Type": "text/plain",
    },
    body: new TextEncoder().encode(JSON.stringify([{title: "Hello, World!"}])),
  }
}
