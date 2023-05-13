package tailcall.runtime.internal

import zio.{Chunk, NonEmptyChunk, Task, ZIO}

sealed trait TValid[+E, +A] {
  self =>
  def <>[E1, A1 >: A](other: => TValid[E1, A1]): TValid[E1, A1] = self orElse other

  def errors: Chunk[E] = fold(_.toChunk, _ => Chunk.empty)

  def flatMap[E1 >: E, B](ab: A => TValid[E1, B]): TValid[E1, B] = self.fold(TValid.fail(_), ab)

  def fold[B](isError: NonEmptyChunk[E] => B, isSucceed: A => B): B =
    self match {
      case TValid.Errors(errors) => isError(errors)
      case TValid.Succeed(value) => isSucceed(value)
    }

  def get(implicit ev: E <:< Nothing): A =
    self match {
      case TValid.Succeed(value) => value
      case TValid.Errors(_)      => throw new NoSuchElementException("Failure does not exist")
    }

  def getOrElse[A1 >: A](a: => A1): A1 = self.getOrElseWith(_ => a)

  def getOrElseWith[A1 >: A](orElse: NonEmptyChunk[E] => A1): A1 = self.fold[A1](orElse, identity)

  def getOrThrow(implicit ev: E <:< String): A =
    self.getOrElseWith(e => throw new RuntimeException(e.mkString("[", ", ", "]")))

  def getOrThrow(prefix: String)(implicit ev: E <:< String): A =
    self.getOrElseWith(e => throw new RuntimeException(prefix + e.mkString("[", ", ", "]")))

  def isEmpty: Boolean = self.fold(_ => true, _ => false)

  def map[B](ab: A => B): TValid[E, B] = self.flatMap(a => TValid.succeed(ab(a)))

  def mapError[E1](f: E => E1): TValid[E1, A] = self.fold(errors => TValid.fail(errors.map(f)), TValid.succeed(_))

  def nonEmpty: Boolean = !isEmpty

  def orElse[E1, A1 >: A](other: => TValid[E1, A1]): TValid[E1, A1] =
    self.fold[TValid[E1, A1]](_ => other, TValid.succeed(_))

  def some: TValid[E, Option[A]] = self.map(Some(_))

  def toEither: Either[NonEmptyChunk[E], A] =
    self match {
      case TValid.Errors(errors) => Left(errors)
      case TValid.Succeed(value) => Right(value)
    }

  def toList: List[A] = self.fold[List[A]](_ => Nil, List(_))

  def toOption: Option[A] = self.fold[Option[A]](_ => None, Some(_))

  def toTask(implicit ev: E <:< String): Task[A] = ZIO.attempt(getOrThrow)

  def toZIO: zio.ZIO[Any, Chunk[E], A] = self.fold(zio.ZIO.fail(_), zio.ZIO.succeed(_))

  def unit: TValid[E, Unit] = map(_ => ())

  def when(cond: Boolean): TValid[E, Unit] =
    self.fold(errors => if (cond) TValid.fail(errors) else TValid.succeed(()), _ => TValid.succeed(()))

  def zip[E1 >: E, B, C](other: TValid[E1, B])(f: (A, B) => C): TValid[E1, C] =
    self.flatMap(a => other.map(b => f(a, b)))

  def zipPar[E1 >: E, B, C](other: TValid[E1, B])(f: (A, B) => C): TValid[E1, C] = {
    (self, other) match {
      case (TValid.Errors(self), TValid.Errors(other)) => TValid.Errors(self ++ other)
      case (TValid.Succeed(a), TValid.Succeed(b))      => TValid.Succeed(f(a, b))
      case (TValid.Errors(self), _)                    => TValid.Errors(self)
      case (_, TValid.Errors(other))                   => TValid.Errors(other)
    }
  }
}

object TValid {
  def fail[E](errors: NonEmptyChunk[E]): TValid[E, Nothing] = Errors(errors)

  def fail[E](head: E, tail: E*): TValid[E, Nothing] = fail(NonEmptyChunk.fromIterable(head, tail.toList))

  def fold[E, A, B](list: List[A], b: B)(f: (B, A) => TValid[E, B]): TValid[E, B] =
    list.foldLeft[TValid[E, B]](succeed(b))((tValid, a) => tValid.flatMap(b => f(b, a)))

  def foreach[A, E, B](list: List[A])(f: A => TValid[E, B]): TValid[E, List[B]] = foreachIterable(list)(f).map(_.toList)

  def foreachChunk[A, E, B](chunk: Chunk[A])(f: A => TValid[E, B]): TValid[E, Chunk[B]] =
    foreachIterable(chunk)(f).map(Chunk.fromIterable(_))

  def foreachIterable[A, E, B](iter: Iterable[A])(f: A => TValid[E, B]): TValid[E, Iterable[B]] = {
    val valuesBuilder = Iterable.newBuilder[B]
    var errorChunk    = Chunk.empty[E]

    iter foreach { a =>
      f(a) match {
        case Errors(errors) => errorChunk = errorChunk ++ errors
        case Succeed(value) => valuesBuilder += value
      }
    }

    errorChunk.nonEmptyOrElse[TValid[E, Iterable[B]]](TValid.succeed(valuesBuilder.result()))(TValid.fail)
  }

  def fromEither[E, A](either: Either[E, A]): TValid[E, A] =
    either.fold[TValid[E, A]](error => fail(NonEmptyChunk.single(error)), succeed(_))

  def fromOption[A](option: Option[A]): TValid[Unit, A] =
    option.fold[TValid[Unit, A]](TValid.fail(Chunk(())))(succeed(_))

  def fromOption[E, A](option: Option[A], error: E): TValid[E, A] =
    option.fold[TValid[E, A]](TValid.fail(error))(succeed(_))

  def none: TValid[Nothing, Option[Nothing]] = succeed(None)

  def some[A](a: A): TValid[Nothing, Option[A]] = succeed(Some(a))

  def succeed[A](value: A): TValid[Nothing, A] = Succeed(value)

  def unit: TValid[Nothing, Unit] = succeed(())

  final case class Errors[E](chunk: NonEmptyChunk[E]) extends TValid[E, Nothing]
  final case class Succeed[A](value: A)               extends TValid[Nothing, A]
}
