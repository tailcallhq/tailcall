package tailcall.remote

import zio.schema.Schema

sealed trait Remote[+A] {}

object Remote {
  def apply[A: Schema](value: A): Remote[A] = ???
}
