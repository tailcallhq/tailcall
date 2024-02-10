class Channel {
  constructor() {
    this._listeners = []
    this._outbox = []
  }

  /**
   * Inserts a listener to the channel.
   * This is called from the client (JS) side.
   */
  addListener(listener) {
    this._listeners.push(listener)
  }

  /**
   * Emits an event on the channel.
   * This is called from the server (Rust) side.
   */
  serverEmit(message) {
    this._listeners.forEach((listener) => listener(message))
    const messages = this._outbox
    this._outbox = []
    return messages
  }

  /**
   * Emits an event on the channel.
   * This is called from the client (JS) side.
   */
  dispatch(message) {
    this._outbox.push(message)
  }
}

globalThis.channel = new Channel()
