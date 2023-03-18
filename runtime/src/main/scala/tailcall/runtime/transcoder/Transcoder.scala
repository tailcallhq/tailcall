package tailcall.runtime.transcoder

/**
 * A transcoder is a function that takes an A and returns a
 * B, or an error. It can be composed using the >>> operator
 * with other transcoders to create a pipeline. A transcoder
 * between A ~> C can be derived provided there exists a B
 * such that a transcoder from A ~> B exists and a
 * transcoder from B ~> C already exists.
 */
final case class Transcoder[-A, +E, +B](run: A => TExit[E, B]) {
  self =>
  def apply(a: A): TExit[E, B] = run(a)

  def >>>[E1 >: E, C](other: Transcoder[B, E1, C]): Transcoder[A, E1, C] = Transcoder(self(_).flatMap(other(_)))
}

object Transcoder {
  def apply[A, E, B](implicit ev: Transcoder[A, E, B]): Transcoder[A, E, B] = ev

  def fromExit[A, E, B](f: A => TExit[E, B]): Transcoder[A, E, B] = Transcoder(f)

  def total[A]: PartiallyAppliedTotal[A] = new PartiallyAppliedTotal[A]()

  final class PartiallyAppliedTotal[A] {
    def apply[E, B](f: A => B): Transcoder[A, E, B] = Transcoder(a => TExit.succeed(f(a)))
  }
}
