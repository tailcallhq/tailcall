package tailcall.runtime.transcoder.value

import caliban.parsing.adt.Type
import tailcall.runtime.internal.TValid
import zio.schema.{Schema, StandardType}

trait ToCalibanType {

  final def toCalibanType(schema: Schema[_], nonNull: Boolean): TValid[String, Type] =
    schema match {
      case record: Schema.Record[_]             => TValid.succeed(Type.NamedType(record.id.name, nonNull))
      case collection: Schema.Collection[_, _]  => collection match {
          case Schema.Sequence(elementSchema, _, _, _, _) => toCalibanType(elementSchema, nonNull = true)
              .map(Type.ListType(_, nonNull))
          case Schema.Map(_, _, _)                        => unsupported("Collection.Map")
          case Schema.Set(_, _)                           => unsupported("Collection.Set")
        }
      case Schema.Lazy(schema)                  => toCalibanType(schema(), nonNull)
      case schema: Schema.Enum[_]               => TValid.succeed(Type.NamedType(schema.id.name, nonNull))
      case Schema.Primitive(standardType, _)    => toCalibanType(standardType, nonNull)
      case Schema.Optional(schema, _)           => toCalibanType(schema, nonNull = true)
      case Schema.Transform(schema, _, _, _, _) => toCalibanType(schema, nonNull)
      case Schema.Fail(_, _)                    => unsupported("Fail")
      case Schema.Tuple2(_, _, _)               => unsupported("Tuple2")
      case Schema.Either(_, _, _)               => unsupported("Either")
      case Schema.Dynamic(_)                    => unsupported("Dynamic")
    }

  def toCalibanType(valueType: StandardType[_], optional: Boolean): TValid[Nothing, Type] =
    TValid.succeed(Type.NamedType(valueType.tag.capitalize, !optional))

  private def unsupported(name: String): TValid[String, Nothing] =
    TValid.fail(s"""Can not convert "$name" to caliban "Type"""")
}
