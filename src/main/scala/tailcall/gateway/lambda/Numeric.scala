package tailcall.gateway.lambda

import zio.schema.DeriveSchema.gen
import zio.schema.Schema

sealed trait Numeric:
  type Input
  def apply(value: Any): Input
  def add(left: Input, right: Input): Input
  def negate(value: Input): Input
  def multiply(left: Input, right: Input): Input
  def divide(left: Input, right: Input): Input
  def modulo(left: Input, right: Input): Input
  def greaterThan(left: Input, right: Input): Boolean
  def greaterThanEqual(left: Input, right: Input): Boolean
  def one: Input
  def schema: Schema[Input]
  // def apply(a: Input): Any ~> Input = Lambda(a)(schema)

// TODO: add more numeric types
object Numeric:
  type Aux[A] = Numeric { type Input = A }
  implicit case object IntTag extends Numeric:
    override type Input = Int
    override def add(left: Int, right: Int): Int                  = left + right
    override def negate(value: Int): Int                          = -value
    override def multiply(left: Int, right: Int): Int             = left * right
    override def divide(left: Int, right: Int): Int               = left / right
    override def modulo(left: Int, right: Int): Int               = left % right
    override def greaterThan(left: Int, right: Int): Boolean      = left > right
    override def greaterThanEqual(left: Int, right: Int): Boolean = left >= right
    override def one: Int                                         = 1
    override def schema: Schema[Int]                              = Schema[Int]
    override def apply(value: Any): Int                           = value.asInstanceOf[Int]
