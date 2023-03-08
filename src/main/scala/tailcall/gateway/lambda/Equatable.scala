package tailcall.gateway.lambda

// TODO: use dependant types instead of Tag
sealed trait Equatable[A] extends Equatable.Tag {
  def equal(a: A, b: A): Boolean
  def any: Equatable[Any] = this.asInstanceOf[Equatable[Any]]
}

object Equatable {
  sealed trait Tag {
    self =>
    final def toEquatable: Equatable[Any] = { self match { case self: Equatable[_] => self.any } }
  }
  implicit case object IntEquatable extends Equatable[Int] {
    override def equal(a: Int, b: Int): Boolean = a == b
  }
}
