package tailcall.runtime.model

import caliban.Value
import caliban.parsing.adt.Directive
import zio.schema.{DynamicValue, Schema}

sealed trait FieldAnnotation {
  self =>
  def toDirective: Blueprint.Directive = FieldAnnotation.toDirective(self)
}

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

  final private def toDirective(annotation: FieldAnnotation): Blueprint.Directive = {
    annotation match {
      case FieldAnnotation.Rename(name) => Blueprint
          .Directive(name = "rename", arguments = Map("name" -> DynamicValue(name)))
    }
  }

  final case class Rename(name: String) extends FieldAnnotation

  implicit def schema: Schema[FieldAnnotation] = ???
}
