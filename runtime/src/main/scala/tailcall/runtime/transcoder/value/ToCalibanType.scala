package tailcall.runtime.transcoder.value

import caliban.parsing.adt.Type
import tailcall.runtime.internal.TValid
import zio.schema.meta.{ExtensibleMetaSchema, MetaSchema}
import zio.schema.{Schema, StandardType}

trait ToCalibanType {

  final def toCalibanType(schema: Schema[_]): TValid[String, Type] = toCalibanType(schema.ast)

  final def toCalibanType(meta: MetaSchema): TValid[String, Type] =
    meta match {
      case ExtensibleMetaSchema.Product(id, _, _, optional)   => TValid.succeed(Type.NamedType(id.name, !optional))
      case ExtensibleMetaSchema.ListNode(item, _, optional)   => toCalibanType(item).map(Type.ListType(_, !optional))
      case ExtensibleMetaSchema.Value(valueType, _, optional) => toCalibanType(valueType, optional)
      case ExtensibleMetaSchema.Tuple(_, _, _, _)             => unsupported("Tuple")
      case ExtensibleMetaSchema.Sum(_, _, _, _)               => unsupported("Sum")
      case ExtensibleMetaSchema.Either(_, _, _, _)            => unsupported("Either")
      case ExtensibleMetaSchema.FailNode(_, _, _)             => unsupported("FailNode")
      case ExtensibleMetaSchema.Dictionary(_, _, _, _)        => unsupported("Dictionary")
      case ExtensibleMetaSchema.Ref(_, _, _)                  => unsupported("Ref")
      case ExtensibleMetaSchema.Known(_, _, _)                => unsupported("Known")
    }

  def toCalibanType(valueType: StandardType[_], optional: Boolean): TValid[Nothing, Type] =
    TValid.succeed(Type.NamedType(valueType.tag.capitalize, !optional))

  private def unsupported(name: String): TValid[String, Nothing] =
    TValid.fail(s"""Can not convert "$name" to caliban "Type"""")
}
