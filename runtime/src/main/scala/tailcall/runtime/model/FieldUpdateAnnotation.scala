package tailcall.runtime.model

import tailcall.runtime.DirectiveCodec
import zio.schema.annotation.caseName
import zio.schema.{DeriveSchema, Schema}

@caseName("update")
final case class FieldUpdateAnnotation(rename: Option[String] = None) {
  def withName(name: String): FieldUpdateAnnotation = copy(rename = Some(name))
}

object FieldUpdateAnnotation {
  def empty: FieldUpdateAnnotation                              = FieldUpdateAnnotation()
  private val schema: Schema[FieldUpdateAnnotation]             = DeriveSchema.gen[FieldUpdateAnnotation]
  implicit val directive: DirectiveCodec[FieldUpdateAnnotation] = DirectiveCodec.fromSchema(schema)
}
