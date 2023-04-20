package tailcall.runtime.internal

import zio.Chunk

sealed trait TValid[+E, +A] {
  self =>
  final def <>[E1, A1 >: A](other: TValid[E1, A1]): TValid[E1, A1] = self orElse other

  final def orElse[E1, A1 >: A](other: TValid[E1, A1]): TValid[E1, A1] =
    self.fold[TValid[E1, A1]](_ => other, TValid.succeed(_))

  final def asThrowable(implicit ev: E <:< String): TValid[Throwable, A] =
    self.fold(err => TValid.fail(new RuntimeException(err)), TValid.succeed(_))

  final def get(implicit ev: E <:< Nothing): A =
    self match {
      case TValid.Failure(_)     => throw new NoSuchElementException("Failure does not exist")
      case TValid.Succeed(value) => value
    }

  final def getOrElse[A1 >: A](orElse: E => A1): A1 = self.fold[A1](orElse, identity)

  final def fold[B](isError: E => B, isSucceed: A => B): B =
    self match {
      case TValid.Failure(message) => isError(message)
      case TValid.Succeed(value)   => isSucceed(value)
    }

  final def some: TValid[E, Option[A]] = self.map(Some(_))

  final def map[B](ab: A => B): TValid[E, B] = self.flatMap(a => TValid.succeed(ab(a)))

  final def flatMap[E1 >: E, B](ab: A => TValid[E1, B]): TValid[E1, B] = self.fold(TValid.fail(_), ab)

  final def toEither: Either[E, A] = self.fold[Either[E, A]](Left(_), Right(_))

  final def toOption: Option[A] = self.fold[Option[A]](_ => None, Some(_))

  final def toList: List[A] = self.fold[List[A]](_ => Nil, List(_))

  final def toZIO: zio.ZIO[Any, E, A] = self.fold(zio.ZIO.fail(_), zio.ZIO.succeed(_))

  final def zip[E1 >: E, B, C](other: TValid[E1, B])(f: (A, B) => C): TValid[E1, C] =
    self.flatMap(a => other.map(b => f(a, b)))
}

object TValid {
  def fold[E, A, B](list: List[A], b: B)(f: (B, A) => TValid[E, B]): TValid[E, B] =
    list.foldLeft[TValid[E, B]](succeed(b))((tValid, a) => tValid.flatMap(b => f(b, a)))

  def foreach[A, E, B](list: List[A])(f: A => TValid[E, B]): TValid[E, List[B]] = foreachIterable(list)(f).map(_.toList)

  def foreachChunk[A, E, B](chunk: Chunk[A])(f: A => TValid[E, B]): TValid[E, Chunk[B]] =
    foreachIterable(chunk)(f).map(Chunk.fromIterable(_))

  def foreachIterable[A, E, B](iter: Iterable[A])(f: A => TValid[E, B]): TValid[E, Iterable[B]] = {
    val builder = Iterable.newBuilder[B]
    iter.foldLeft[TValid[E, Unit]](succeed(()))((acc, a) => acc.flatMap(_ => f(a).map(builder += _)))
      .map(_ => builder.result())
  }

  def fromEither[E, A](either: Either[E, A]): TValid[E, A] = either.fold[TValid[E, A]](fail(_), succeed(_))

  def fail[E](message: E): TValid[E, Nothing] = Failure(message)

  def fromOption[A](option: Option[A]): TValid[Unit, A] = option.fold[TValid[Unit, A]](TValid.fail(()))(Succeed(_))

  def none: TValid[Nothing, Option[Nothing]] = succeed(None)

  def succeed[A](value: A): TValid[Nothing, A] = Succeed(value)

  def some[A](a: A): TValid[Nothing, Option[A]] = succeed(Some(a))

  def unsupported(from: String, to: String): TValid[String, Nothing] =
    fail(s"Conversion from ${from} to ${to} is not yet supported")

  // TODO: can fail with a chunk of errors
  final case class Failure[E](message: E) extends TValid[E, Nothing]
  final case class Succeed[A](value: A)   extends TValid[Nothing, A]
}
