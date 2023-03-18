package tailcall.runtime.transcoder

import tailcall.runtime.transcoder.Transcoder.TExit
import zio.Chunk

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

  def total[A, B](f: A => B): Transcoder[A, Nothing, B] = Transcoder(a => TExit.succeed(f(a)))

  sealed trait TExit[+E, +A] {
    self =>
    def get(implicit ev: E <:< Nothing): A =
      self match {
        case TExit.Failure(_)     => throw new NoSuchElementException("Failure does not exist")
        case TExit.Succeed(value) => value
      }

    def map[B](ab: A => B): TExit[E, B] = self.flatMap(a => TExit.succeed(ab(a)))

    def flatMap[E1 >: E, B](ab: A => TExit[E1, B]): TExit[E1, B] = self.fold(TExit.fail(_), ab)

    def orElse[E1, A1 >: A](other: TExit[E1, A1]): TExit[E1, A1] =
      self.fold[TExit[E1, A1]](_ => other, TExit.succeed(_))

    def <>[E1, A1 >: A](other: TExit[E1, A1]): TExit[E1, A1] = self orElse other

    def toEither: Either[E, A] = self.fold[Either[E, A]](Left(_), Right(_))

    def toOption: Option[A] = self.fold[Option[A]](_ => None, Some(_))

    def fold[B](isError: E => B, isSucceed: A => B): B =
      self match {
        case TExit.Failure(message) => isError(message)
        case TExit.Succeed(value)   => isSucceed(value)
      }

    def getOrElse[A1 >: A](default: => A1): A1 = self.fold[A1](_ => default, identity)
  }

  object TExit {
    def fail[E](message: E): TExit[E, Nothing] = Failure(message)

    def succeed[A](value: A): TExit[Nothing, A] = Succeed(value)

    def fromOption[A](option: Option[A]): TExit[Unit, A] = option.fold[TExit[Unit, A]](TExit.fail(()))(Succeed(_))

    def foreach[A, E, B](list: List[A])(f: A => TExit[E, B]): TExit[E, List[B]] = foreachIterable(list)(f).map(_.toList)

    def foreachChunk[A, E, B](chunk: Chunk[A])(f: A => TExit[E, B]): TExit[E, Chunk[B]] =
      foreachIterable(chunk)(f).map(Chunk.fromIterable(_))

    def foreachIterable[A, E, B](iter: Iterable[A])(f: A => TExit[E, B]): TExit[E, Iterable[B]] = {
      val builder = Iterable.newBuilder[B]
      iter.foldLeft[TExit[E, Unit]](succeed(()))((acc, a) => acc.flatMap(_ => f(a).map(builder += _)))
        .map(_ => builder.result())
    }

    def fromEither[E, A](either: Either[E, A]): TExit[E, A] = either.fold[TExit[E, A]](fail(_), succeed(_))

    final case class Failure[E](message: E) extends TExit[E, Nothing]

    final case class Succeed[A](value: A) extends TExit[Nothing, A]
  }

  implicit final class Syntax[A](private val a: A) {
    def transcode[B](implicit transcoder: Transcoder[A, Nothing, B]): B = transcoder.run(a).get

    def transcodeOrFailWith[B, E](implicit transcoder: Transcoder[A, E, B]): Either[E, B] = transcoder.run(a).toEither

    def transcodeOrDefault[B](b: => B)(implicit transcoder: Transcoder[A, _, B]): B = transcoder.run(a).getOrElse(b)
  }
}
