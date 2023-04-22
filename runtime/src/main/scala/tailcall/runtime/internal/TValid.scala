package tailcall.runtime.internal

import zio.Chunk

final case class TValid[+E, +A](result: Either[E, A]) {
  self =>
  def <>[E1, A1 >: A](other: TValid[E1, A1]): TValid[E1, A1] = self orElse other

  def orElse[E1, A1 >: A](other: TValid[E1, A1]): TValid[E1, A1] =
    self.fold[TValid[E1, A1]](_ => other, TValid.succeed(_))

  def asThrowable(implicit ev: E <:< String): TValid[Throwable, A] =
    self.fold(err => TValid.fail(new RuntimeException(err)), TValid.succeed(_))

  def get(implicit ev: E <:< Nothing): A =
    self.result match {
      case Right(value) => value
      case Left(_)      => throw new NoSuchElementException("Failure does not exist")
    }

  def getOrElse[A1 >: A](orElse: E => A1): A1 = self.fold[A1](orElse, identity)

  def fold[B](isError: E => B, isSucceed: A => B): B = self.result.fold(isError, isSucceed)

  def some: TValid[E, Option[A]] = self.map(Some(_))

  def map[B](ab: A => B): TValid[E, B] = self.flatMap(a => TValid.succeed(ab(a)))

  def flatMap[E1 >: E, B](ab: A => TValid[E1, B]): TValid[E1, B] = self.fold(TValid.fail(_), ab)

  def toEither: Either[E, A] = self.fold[Either[E, A]](Left(_), Right(_))

  def toOption: Option[A] = self.fold[Option[A]](_ => None, Some(_))

  def toList: List[A] = self.fold[List[A]](_ => Nil, List(_))

  def toZIO: zio.ZIO[Any, E, A] = self.fold(zio.ZIO.fail(_), zio.ZIO.succeed(_))

  def zip[E1 >: E, B, C](other: TValid[E1, B])(f: (A, B) => C): TValid[E1, C] =
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

  def fail[E](message: E): TValid[E, Nothing] = TValid(Left(message))

  def fromOption[A](option: Option[A]): TValid[Unit, A] = option.fold[TValid[Unit, A]](TValid.fail(()))(succeed(_))

  def none: TValid[Nothing, Option[Nothing]] = succeed(None)

  def succeed[A](value: A): TValid[Nothing, A] = TValid(Right(value))

  def some[A](a: A): TValid[Nothing, Option[A]] = succeed(Some(a))
}
