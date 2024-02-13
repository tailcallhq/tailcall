class Channel {
  constructor() {
    this._listeners = {}
    this._outbox = []
    this._id = 0
  }

  /**
   * Inserts a listener to the channel.
   * This is called from the client (JS) side.
   */
  addListener(listener) {
    const id = this.get_id()
    this._listeners[id] = listener
    return id
  }

  removeListener(id) {
    delete this._listeners[id]
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

  /**
   * Creates a unique id for a message.
   * TODO: move to a separate module.
   */
  get_id() {
    return this._id++
  }
}

globalThis.channel = new Channel()
