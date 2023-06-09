package tailcall.runtime.internal

import tailcall.runtime.internal.TValid.Cause
import zio.{Chunk, NonEmptyChunk, Task, ZIO}

sealed trait TValid[+E, +A] {
  self =>
  final def |[E1, A1 >: A](other: => TValid[E1, A1]): TValid[E1, A1] = self orElse other

  final def <>[E1, A1 >: A](other: => TValid[E1, A1]): TValid[E1, A1] = self orElse other

  final def as[B](b: => B): TValid[E, B] = self.map(_ => b)

  final def errors: Chunk[TValid.Cause[E]] = fold(_.toChunk, _ => Chunk.empty)

  final def flatMap[E1 >: E, B](ab: A => TValid[E1, B]): TValid[E1, B] = self.fold(TValid.failCause(_), ab)

  final def fold[B](isError: NonEmptyChunk[TValid.Cause[E]] => B, isSucceed: A => B): B =
    self match {
      case TValid.Errors(errors) => isError(errors)
      case TValid.Succeed(value) => isSucceed(value)
    }

  final def get(implicit ev: E <:< Nothing): A =
    self match {
      case TValid.Succeed(value) => value
      case TValid.Errors(_)      => throw new NoSuchElementException("Failure does not exist")
    }

  final def getOrElse[A1 >: A](a: => A1): A1 = self.getOrElseWith(_ => a)

  final def getOrElseWith[A1 >: A](orElse: NonEmptyChunk[Cause[E]] => A1): A1 = self.fold[A1](orElse, identity)

  final def isInvalid: Boolean = !isValid

  final def isValid: Boolean = self.fold(_ => false, _ => true)

  final def map[B](ab: A => B): TValid[E, B] = self.flatMap(a => TValid.succeed(ab(a)))

  final def mapError[E1](f: E => E1): TValid[E1, A] =
    self
      .fold(errors => TValid.failCause(errors.map(cause => cause.copy(message = f(cause.message)))), TValid.succeed(_))

  final def orElse[E1, A1 >: A](other: => TValid[E1, A1]): TValid[E1, A1] =
    self.fold[TValid[E1, A1]](_ => other, TValid.succeed(_))

  final def some: TValid[E, Option[A]] = self.map(Some(_))

  final def toEither: Either[NonEmptyChunk[Cause[E]], A] =
    self match {
      case TValid.Errors(errors) => Left(errors)
      case TValid.Succeed(value) => Right(value)
    }

  final def toList: List[A] = self.fold[List[A]](_ => Nil, List(_))

  final def toOption: Option[A] = self.fold[Option[A]](_ => None, Some(_))

  final def toTask(implicit ev: E <:< String): Task[A] = ZIO.attempt(unwrap)

  final def toZIO: zio.ZIO[Any, Chunk[Cause[E]], A] = self.fold(zio.ZIO.fail(_), zio.ZIO.succeed(_))

  final def trace(paths: String*): TValid[E, A] =
    self match {
      case TValid.Errors(chunk) => TValid.Errors(chunk.map(cause => cause.copy(trace = paths.toList ++ cause.trace)))
      case self                 => self
    }

  final def unit: TValid[E, Unit] = map(_ => ())

  final def unless(cond: Boolean): TValid[E, Unit] = when(!cond)

  final def unwrap(implicit ev: E <:< String): A = unwrapWith("")

  final def unwrapWith(prefix: String)(implicit ev: E <:< String): A =
    self.getOrElseWith(e =>
      throw new RuntimeException(prefix + e.map(cause => {
        val str = if (cause.trace.isEmpty) "" else cause.trace.mkString("[", ", ", "]") + ": "
        s"${str}${cause.message}"
      }).mkString("[", ", ", "]"))
    )

  final def when(cond: Boolean): TValid[E, Unit] =
    self.fold(errors => if (cond) TValid.failCause(errors) else TValid.succeed(()), _ => TValid.succeed(()))

  final def zip[E1 >: E, B, C](other: TValid[E1, B])(f: (A, B) => C): TValid[E1, C] =
    self.flatMap(a => other.map(b => f(a, b)))

  final def zipPar[E1 >: E, B, C](other: TValid[E1, B])(f: (A, B) => C): TValid[E1, C] = {
    (self, other) match {
      case (TValid.Errors(self), TValid.Errors(other)) => TValid.Errors(self ++ other)
      case (TValid.Succeed(a), TValid.Succeed(b))      => TValid.Succeed(f(a, b))
      case (TValid.Errors(self), _)                    => TValid.Errors(self)
      case (_, TValid.Errors(other))                   => TValid.Errors(other)
    }
  }
}

object TValid {
  def all[E, A](seq: TValid[E, A]*): TValid[E, A] = seq.reduce(_ <> _)

  def fail[E](errors: NonEmptyChunk[E]): TValid[E, Nothing] = Errors(errors.map(Cause(_)))

  def fail[E](head: E, tail: E*): TValid[E, Nothing] = fail(NonEmptyChunk.fromIterable(head, tail.toList))

  def failCause[E](errors: NonEmptyChunk[Cause[E]]): TValid[E, Nothing] = Errors(errors)

  def fold[E, A, B](list: List[A], b: B)(f: (B, A) => TValid[E, B]): TValid[E, B] =
    list.foldLeft[TValid[E, B]](succeed(b))((tValid, a) => tValid.flatMap(b => f(b, a)))

  def foreach[A, E, B](list: List[A])(f: A => TValid[E, B]): TValid[E, List[B]] = foreachIterable(list)(f).map(_.toList)

  def foreachChunk[A, E, B](chunk: Chunk[A])(f: A => TValid[E, B]): TValid[E, Chunk[B]] =
    foreachIterable(chunk)(f).map(Chunk.fromIterable(_))

  def foreachDiscard[A, E, B](list: List[A])(f: A => TValid[E, B]): TValid[E, Unit] = foreach(list)(f).unit

  def foreachIterable[A, E, B](iter: Iterable[A])(f: A => TValid[E, B]): TValid[E, Iterable[B]] = {
    val valuesBuilder = Iterable.newBuilder[B]
    var errorChunk    = Chunk.empty[Cause[E]]

    iter foreach { a =>
      f(a) match {
        case Errors(errors) => errorChunk = errorChunk ++ errors
        case Succeed(value) => valuesBuilder += value
      }
    }

    errorChunk.nonEmptyOrElse[TValid[E, Iterable[B]]](TValid.succeed(valuesBuilder.result()))(TValid.failCause(_))
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

  final case class Errors[+E](chunk: NonEmptyChunk[Cause[E]]) extends TValid[E, Nothing]
  final case class Succeed[+A](value: A)                      extends TValid[Nothing, A]
  final case class Cause[+E](message: E, trace: List[String] = Nil)
}
