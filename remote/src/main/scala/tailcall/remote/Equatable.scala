package tailcall.remote

sealed trait Equatable[A] {
  self =>
  def tag: Equatable.Tag
  def equal(a: A, b: A): Boolean
}

object Equatable {
  sealed trait Tag {
    self =>
    def toEquatable: Equatable[Any] = Equatable.fromTag(self).asInstanceOf[Equatable[Any]]
  }

  object Tag {
    case object IntTag extends Tag
  }

  def fromTag(tag: Tag): Equatable[_] = tag match { case Tag.IntTag => IntEquatable }

  implicit object IntEquatable extends Equatable[Int] {
    override def tag: Tag                       = Tag.IntTag
    override def equal(a: Int, b: Int): Boolean = a == b
  }
}
