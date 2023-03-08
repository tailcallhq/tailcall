package tailcall.gateway.lambda

import zio.schema.DeriveSchema.gen
import zio.schema.Schema

// TODO: use dependant types instead of Tag
sealed trait Numeric[A] extends Numeric.Tag {
  def add(left: A, right: A): A
  def negate(value: A): A
  def multiply(left: A, right: A): A
  def divide(left: A, right: A): A
  def modulo(left: A, right: A): A
  def greaterThan(left: A, right: A): Boolean
  def greaterThanEqual(left: A, right: A): Boolean
  def one: A
  final def any: Numeric[Any] = this.asInstanceOf[Numeric[Any]]
  def schema: Schema[A]
  def apply(a: A): Any ~> A   = Lambda(a)(schema)
}

// TODO: add more numeric types
object Numeric {
  sealed trait Tag {
    self =>
    def numeric: Numeric[Any] = self match { case self: Numeric[_] => self.any }
  }

  implicit case object IntTag extends Numeric[Int] {
    override def add(left: Int, right: Int): Int                  = left + right
    override def negate(value: Int): Int                          = -value
    override def multiply(left: Int, right: Int): Int             = left * right
    override def divide(left: Int, right: Int): Int               = left / right
    override def modulo(left: Int, right: Int): Int               = left % right
    override def greaterThan(left: Int, right: Int): Boolean      = left > right
    override def greaterThanEqual(left: Int, right: Int): Boolean = left >= right
    override def one: Int                                         = 1

    override def schema: Schema[Int] = Schema[Int]
  }
}
