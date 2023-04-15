package tailcall.runtime.model

import tailcall.runtime.DirectiveCodec
import zio.schema.annotation.caseName
import zio.schema.{DeriveSchema, Schema}

@caseName("modify")
final case class ModifyField(rename: Option[String] = None, omit: Option[Boolean] = None) {
  self =>
  def withName(name: String): ModifyField  = copy(rename = Some(name))
  def withOmit(omit: Boolean): ModifyField = copy(omit = Some(omit))
  def nonEmpty: Boolean                    = rename.nonEmpty

  def mergeRight(other: ModifyField): ModifyField = {
    val rename = other.rename.orElse(self.rename)
    val omit   = other.omit.orElse(self.omit)
    ModifyField(rename, omit)
  }
}

object ModifyField {
  def empty: ModifyField                              = ModifyField()
  private val schema: Schema[ModifyField]             = DeriveSchema.gen[ModifyField]
  implicit val directive: DirectiveCodec[ModifyField] = DirectiveCodec.fromSchema(schema)
}
