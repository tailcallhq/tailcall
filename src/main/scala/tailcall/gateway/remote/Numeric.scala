package tailcall.gateway.remote

sealed trait Numeric[A] {
  def add(left: A, right: A): A
  def negate(value: A): A
  def multiply(left: A, right: A): A
  def divide(left: A, right: A): A
  def modulo(left: A, right: A): A
  def greaterThan(left: A, right: A): Boolean
  def one: A
  def any: Numeric[Any] = this.asInstanceOf[Numeric[Any]]
}

// TODO: add more numeric types
object Numeric {

  implicit case object IntTag extends Numeric[Int] {
    override def add(left: Int, right: Int): Int             = left + right
    override def negate(value: Int): Int                     = -value
    override def multiply(left: Int, right: Int): Int        = left * right
    override def divide(left: Int, right: Int): Int          = left / right
    override def modulo(left: Int, right: Int): Int          = left % right
    override def greaterThan(left: Int, right: Int): Boolean = left > right
    override def one: Int                                    = 1
  }
}
