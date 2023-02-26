package tailcall.gateway.lambda

import tailcall.gateway.ast.Context
import zio.schema.DynamicValue

trait Accessor[B, P] {
  def get(b: B): P
  def set(b: B, p: P): B
}
object Accessor      {

  implicit def fieldAccessor: Accessor[Context, DynamicValue] =
    new Accessor[Context, DynamicValue] {
      self =>
      override def get(b: Context): DynamicValue = ???
      override def set(b: Context, p: DynamicValue): Context = ???
    }
}
