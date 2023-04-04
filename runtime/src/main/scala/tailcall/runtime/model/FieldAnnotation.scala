package tailcall.runtime.model

import caliban.Value
import caliban.parsing.adt.Directive
import tailcall.runtime.DirectiveCodec
import zio.schema.{DeriveSchema, Schema}

sealed trait FieldAnnotation

object FieldAnnotation {
  def from(directives: List[Directive]): List[FieldAnnotation] = directives.flatMap(from(_))

  def from(directive: Directive): Option[FieldAnnotation] =
    directive.name match {
      case "rename" => directive.arguments.get("name") match {
          case Some(Value.StringValue(value)) => Some(Rename(value))
          case _                              => None
        }
      case _        => None
    }

  def rename(name: String): FieldAnnotation = Rename(name)

  final case class Rename(name: String) extends FieldAnnotation

  implicit val schema: Schema[FieldAnnotation]                 = DeriveSchema.gen[FieldAnnotation]
  implicit val directiveCodec: DirectiveCodec[FieldAnnotation] = DirectiveCodec.fromSchema(schema)
}
