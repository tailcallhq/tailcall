package tailcall.runtime.model

import caliban.parsing.adt.Definition.TypeSystemDefinition.DirectiveLocation
import tailcall.runtime.{DirectiveCodec, DirectiveDefinitionBuilder}
import zio.schema.annotation.caseName
import zio.schema.{DeriveSchema, Schema}

@caseName("modify")
final case class ModifyField(name: Option[String] = None, omit: Option[Boolean] = None) {
  self =>
  def withName(name: String): ModifyField  = copy(name = Some(name))
  def withOmit(omit: Boolean): ModifyField = copy(omit = Some(omit))
  def isEmpty: Boolean                     = name.isEmpty && omit.isEmpty
  def nonEmpty: Boolean                    = !isEmpty

  def mergeRight(other: ModifyField): ModifyField = {
    val rename = other.name.orElse(self.name)
    val omit   = other.omit.orElse(self.omit)

    ModifyField(rename, omit)
  }
}

object ModifyField {
  def empty: ModifyField                                           = ModifyField()
  implicit val schema: Schema[ModifyField]                         = DeriveSchema.gen[ModifyField]
  implicit val directive: DirectiveCodec[ModifyField]              = DirectiveCodec.fromSchema(schema)
  def directiveDefinition: DirectiveDefinitionBuilder[ModifyField] =
    DirectiveDefinitionBuilder.make[ModifyField].withLocations(
      DirectiveLocation.TypeSystemDirectiveLocation.FIELD_DEFINITION,
      DirectiveLocation.TypeSystemDirectiveLocation.INPUT_FIELD_DEFINITION,
    )
}
