package tailcall.gateway

import zio.ZIO

/**
 * A composable domain that can take an input of type A and
 * validate it producing output values of type B. The in the
 * process of validation, it can also produce traces or logs
 */
sealed trait Validation[-R, +E, -A, +B] {
  self =>
  import Validation._

  final def ++[R1 <: R, E1 >: E, A1 <: A, B1 >: B](
    other: Validation[R1, E1, A1, B1],
  ): Validation[R1, E1, A1, B1] = {
    Combine(self, other)
  }

  final def flatMap[R1 <: R, E1 >: E, A1 <: A, B1](
    f: B => Validation[R1, E1, A1, B1],
  ): Validation[R1, E1, A1, B1] = {
    FlatMap(self, f)
  }

  final def map[B1](f: B => B1): Validation[R, E, A, B1] = {
    self.flatMap(b => Validation.value(f(b)))
  }

  def validate[A1 <: A](a: A1): ZIO[R, E, Status[B]] = {
    Validation.validate(self, a)
  }

  def eval: Eval[R, E, A, B] = Eval(self)
}

object Validation {
  final case class Eval[-R, +E, -A, +B](validation: Validation[R, E, A, B]) {
    def validate(a: A): ZIO[R, E, Status[B]]  = Validation.validate(validation, a)
    def values(a: A): ZIO[R, E, List[B]]      = validate(a).map(_.values)
    def traces(a: A): ZIO[R, E, List[String]] = validate(a).map(_.traces)
  }

  final case class Combine[R, E, A, B](self: Validation[R, E, A, B], other: Validation[R, E, A, B])
      extends Validation[R, E, A, B]
  final case class Value[R, B](value: B)     extends Validation[R, Nothing, Any, B]
  final case class Trace[R](message: String) extends Validation[R, Nothing, Any, Nothing]
  final case class Access[R, E, A, B](f: A => Validation[R, E, Any, B])
      extends Validation[R, E, A, B]
  final case class FlatMap[R, E, A, B, C](
    self: Validation[R, E, A, B],
    f: B => Validation[R, E, A, C],
  ) extends Validation[R, E, A, C]
  case object Empty                          extends Validation[Any, Nothing, Any, Nothing]
  case class FromZIO[R, E, A, B](zio: A => ZIO[R, E, B]) extends Validation[R, E, A, B]

  // Constructors
  def value[B](value: B): Validation[Any, Nothing, Any, B] = {
    Value(value)
  }

  def trace(message: String): Validation[Any, Nothing, Any, Nothing] = {
    Trace(message)
  }

  def access[A]: Validation[Any, Nothing, A, A] = {
    Access[Any, Nothing, A, A](Validation.value(_))
  }

  def empty: Validation[Any, Nothing, Any, Nothing] = {
    Empty
  }

  def fromZIO[R, A, E, B](zio: A => ZIO[R, E, B]): Validation[R, E, A, B] = FromZIO(zio)

  final case class Status[+A](traces: List[String], values: List[A]) {
    self =>
    def ++[A1 >: A](other: Status[A1]): Status[A1] = {
      Status(self.traces ++ other.traces, self.values ++ other.values)
    }

    def flatMap[B](f: A => Status[B]): Status[B] = {
      val result = self.values.map(f(_))
      val traces = result.flatMap(_.traces)
      val values = result.flatMap(_.values)
      Status(self.traces ++ traces, values)
    }

    def map[B](f: A => B): Status[B] = {
      self.flatMap(a => Status.value(f(a)))
    }
  }

  object Status {
    def trace(message: String): Status[Nothing]       = trace(List(message))
    def trace(message: List[String]): Status[Nothing] = Status(message, Nil)
    def value[A](a: A): Status[A]                     = value(List(a))
    def value[A](a: List[A]): Status[A]               = Status(Nil, a)
    def empty: Status[Nothing]                        = Status(Nil, Nil)
  }

  def validate[R, A, E, B](self: Validation[R, E, A, B], a: A): ZIO[R, E, Status[B]] = {
    self match {
      case Combine(first, second) =>
        for {
          firstStatus  <- first.validate(a)
          secondStatus <- second.validate(a)
        } yield firstStatus ++ secondStatus

      case Value(value) =>
        ZIO.succeed(Status.value(value))

      case Trace(message) =>
        ZIO.succeed(Status.trace(message))

      case Access(f) =>
        f(a).validate(a)

      case FlatMap(v, f) =>
        for {
          firstStatus  <- v.validate(a)
          secondStatus <- ZIO
            .foreachPar(firstStatus.values)(f(_).validate(a))
            .map(_.fold[Status[B]](Status.empty)(_ ++ _))
        } yield Status(firstStatus.traces ++ secondStatus.traces, secondStatus.values)

      case FromZIO(f) =>
        f(a).map(Status.value(_))

      case Empty =>
        ZIO.succeed(Status.empty)
    }
  }
}
