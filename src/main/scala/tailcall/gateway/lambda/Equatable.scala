package tailcall.gateway.lambda

sealed trait Equatable[A]:
  def equal(a: A, b: A): Boolean
  def any: Equatable[Any] = this.asInstanceOf[Equatable[Any]]

object Equatable:
  implicit case object IntEquatable extends Equatable[Int]:
    override def equal(a: Int, b: Int): Boolean = a == b
