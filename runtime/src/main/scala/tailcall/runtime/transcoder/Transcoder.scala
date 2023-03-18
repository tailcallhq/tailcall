package tailcall.runtime.transcoder

/**
 * A transcoder is a function that takes an A and returns a
 * B, or an error. It can be composed using the >>> operator
 * with other transcoders to create a pipeline. A transcoder
 * between A ~> C can be derived provided there exists a B
 * such that a transcoder from A ~> B exists and a
 * transcoder from B ~> C already exists.
 */
final case class Transcoder[-A, +B](run: A => TExit[B]) {
  self =>
  def apply(a: A): TExit[B] = run(a)

  def >>>[C](other: Transcoder[B, C]): Transcoder[A, C] = Transcoder(self(_).flatMap(other(_)))
}

object Transcoder {
  def apply[A, B](implicit ev: Transcoder[A, B]): Transcoder[A, B] = ev

  def collect[A]: PartiallyAppliedCollect[A] = new PartiallyAppliedCollect[A]()

  def collectEither[A]: PartiallyAppliedCollectEither[A] = new PartiallyAppliedCollectEither[A]()

  def fromExit[A, B](f: A => TExit[B]): Transcoder[A, B] = Transcoder(f)

  def total[A]: PartiallyAppliedTotal[A] = new PartiallyAppliedTotal[A]()

  final class PartiallyAppliedCollect[A] {
    def apply[B](pf: PartialFunction[A, B]): Transcoder[A, B] = Transcoder(a => TExit.fromOption(pf.lift(a)))
  }

  final class PartiallyAppliedCollectEither[A] {
    def apply[B](pf: PartialFunction[A, Either[String, B]]): Transcoder[A, B] =
      Transcoder(a =>
        pf.lift(a) match {
          case Some(value) => TExit.fromEither(value)
          case None        => TExit.empty
        }
      )
  }

  final class PartiallyAppliedTotal[A] {
    def apply[B](f: A => B): Transcoder[A, B] = Transcoder(a => TExit.succeed(f(a)))
  }
}
