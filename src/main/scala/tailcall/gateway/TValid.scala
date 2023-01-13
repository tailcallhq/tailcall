package tailcall.gateway

import zio.Chunk

final case class TValid[+E, +A](errors: Chunk[E], values: Chunk[A]) {
  self =>
  def flatMap[E1 >: E, B](f: A => TValid[E1, B]): TValid[E1, B] = {
    val validation = values.map(f(_))
    val failures0  = validation.flatMap(_.errors)
    val successes0 = validation.flatMap(_.values)
    TValid(errors ++ failures0, successes0)
  }

  def map[B](f: A => B): TValid[E, B] = { flatMap(a => TValid.success(f(a))) }

  def ++[E1 >: E, A1 >: A](other: TValid[E1, A1]): TValid[E1, A1] = {
    TValid(self.errors ++ other.errors, self.values ++ other.values)
  }
}

object TValid {
  def success[A](a: A): TValid[Nothing, A] = TValid(Chunk.empty, Chunk.single(a))
  def fail[E](e: E): TValid[E, Nothing]    = TValid(Chunk.single(e), Chunk.empty)
  def empty: TValid[Nothing, Nothing]      = TValid(Chunk.empty, Chunk.empty)
  def unit: TValid[Nothing, Unit]          = success(())
  def from[E, A](iterable: Iterable[TValid[E, A]]): TValid[E, A] = {
    val failures0  = Chunk.fromIterable(iterable).flatMap(_.errors)
    val successes0 = Chunk.fromIterable(iterable).flatMap(_.values)
    TValid(failures0, successes0)
  }
}
