package tailcall.runtime.model

import zio.schema.DynamicValue

sealed trait FieldAnnotation {
  self =>
  def toDirective: Blueprint.Directive = FieldAnnotation.toDirective(self)
}

object FieldAnnotation {
  final case class Rename(name: String) extends FieldAnnotation

  def rename(name: String): FieldAnnotation = Rename(name)

  final private def toDirective(annotation: FieldAnnotation): Blueprint.Directive = {
    annotation match {
      case FieldAnnotation.Rename(name) => Blueprint
          .Directive(name = "rename", arguments = Map("name" -> DynamicValue(name)))
    }
  }
}
