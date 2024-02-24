export interface EventRequest {
  url: string
  method: string
  headers: {[key: string]: string}
}

export interface CommandRequest extends EventRequest {
  body?: ArrayBuffer | Uint8Array | string
}

export interface Response {
  status: number
  headers: {[key: string]: string}
  body?: ArrayBuffer | Uint8Array
}

export type Event = {request: EventRequest}
export type Command = {request: CommandRequest} | {response: Response}
