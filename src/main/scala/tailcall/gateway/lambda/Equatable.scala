package tailcall.gateway.lambda

sealed trait Equatable[-A] extends Equatable.Tag:
  self =>

  def tag: Equatable.Tag      = this.asInstanceOf[Equatable.Tag]
  def eq(a: A, b: A): Boolean =
    self match
      case Equatable.IntEquatable => a.asInstanceOf[Int] == b.asInstanceOf[Int]

object Equatable:
  sealed trait Tag
  implicit case object IntEquatable extends Equatable[Int]
