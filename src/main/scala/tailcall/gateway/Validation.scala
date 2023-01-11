package tailcall.gateway

import tailcall.gateway.Validation._

/**
 * A composable domain that can take an input of type A and
 * validate it producing output values of type B. The in the
 * process of validation, it can also produce traces or logs
 */
final case class Validation[-A, +B] private[Validation] (validate: A => Status[B]) {
  self =>
  import Validation._

  def ++[A1 <: A, B1 >: B](other: Validation[A1, B1]): Validation[A1, B1] = {
    Validation { a =>
      self.validate(a) ++ other.validate(a)
    }
  }

  def flatMap[A1 <: A, B1](f: B => Validation[A1, B1]): Validation[A1, B1] = {
    Validation { a =>
      self.validate(a).flatMap(b => f(b).validate(a))
    }
  }

  def map[B1](f: B => B1): Validation[A, B1] = {
    self.flatMap(b => value(f(b)))
  }

  def contramap[A1](f: A1 => A): Validation[A1, B] = {
    Validation(a1 => self.validate(f(a1)))
  }
}

object Validation {
  def value[B](value: B): Validation[Any, B] = {
    Validation(_ => Status.value(value))
  }

  def trace(messages: String): Validation[Any, Nothing] = {
    Validation(_ => Status.trace(messages))
  }

  def make[A]: PartialTypeChecker[A] = new PartialTypeChecker(())

  def access[A]: Validation[A, A] = make[A](Status.value(_))

  final class PartialTypeChecker[A](val unit: Unit) {
    def apply[B](f: A => Status[B]): Validation[A, B] = {
      Validation(f)
    }
  }

  val empty: Validation[Any, Nothing] = {
    Validation(_ => Status.empty)
  }

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
}
