package tailcall.runtime.model

import caliban.parsing.adt.Directive
import tailcall.runtime.DirectiveCodec
import tailcall.runtime.DirectiveCodec.DecoderSyntax
import zio.schema.DeriveSchema
import zio.schema.annotation.caseName

sealed trait FieldAnnotation

object FieldAnnotation {
  def from(directives: List[Directive]): List[FieldAnnotation] = directives.flatMap(from(_))

  def from(directive: Directive): Option[FieldAnnotation] = directive.fromDirective[Rename].toOption

  def rename(name: String): FieldAnnotation = Rename(name)

  @caseName("rename")
  final case class Rename(name: String) extends FieldAnnotation

  object Rename {
    implicit val codec: DirectiveCodec[Rename] = DirectiveCodec.fromSchema(DeriveSchema.gen[Rename])
  }
}
