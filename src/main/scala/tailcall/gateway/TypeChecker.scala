package tailcall.gateway

import zio._
import tailcall.gateway.adt.Config
import caliban.parsing.adt.Document

import TypeChecker._

final case class TypeChecker[-A](validate: A => Status) {
  self =>
  def ++[A1 <: A](other: TypeChecker[A1]): TypeChecker[A1] = TypeChecker[A1](a =>
    self.validate(a) ++ other.validate(a),
  )

  def contramap[A1](f: A1 => A): TypeChecker[A1] = TypeChecker[A1](a => self.validate(f(a)))
}

object TypeChecker {
  sealed trait Cause
  final case class Message(message: String) extends Cause

  sealed trait Status {
    self =>
    def ++(other: Status): Status = Status.Combine(self, other)
  }

  object Status {
    case object Empty                                     extends Status
    final case class Error(cause: Cause)                  extends Status
    final case class Combine(self: Status, other: Status) extends Status
  }

  def empty: TypeChecker[Any] = TypeChecker(_ => Status.Empty)

  def error(message: String): TypeChecker[Any] = TypeChecker[Any](_ =>
    Status.Error(Message(message)),
  )

}
