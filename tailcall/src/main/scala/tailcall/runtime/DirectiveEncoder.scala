package tailcall.runtime

import caliban.InputValue
import caliban.parsing.adt.Directive
import tailcall.runtime.internal.TValid
import zio.json.{DecoderOps, JsonEncoder}
import zio.schema.Schema
import zio.schema.annotation.caseName

final case class DirectiveEncoder[A](name: String, encode: A => TValid[String, Directive]) {
  def contramap[B](ab: B => A): DirectiveEncoder[B] = DirectiveEncoder(name, ab andThen encode)
  def withName(name: String): DirectiveEncoder[A]   = copy(name = name)
}

object DirectiveEncoder {
  implicit def encoder[A](implicit codec: DirectiveCodec[A]): DirectiveEncoder[A] = codec.encoder

  def fromJsonEncoder[A](directiveName: String, encoder: JsonEncoder[A]): DirectiveEncoder[A] =
    DirectiveEncoder(
      directiveName,
      a =>
        TValid.fromEither(encoder.encodeJson(a).fromJson[Map[String, InputValue]])
          .map(args => Directive(directiveName, args)),
    )

  def fromSchema[A](schema: Schema[A]): DirectiveEncoder[A] = {
    val jsonEncoder = zio.schema.codec.JsonCodec.jsonEncoder(schema)
    val nameHint    = schema.annotations.collectFirst { case caseName(name) => name }
    val schemaName  = schema match {
      case schema: Schema.Enum[_]   => schema.id.name
      case schema: Schema.Record[_] => schema.id.name
      case _                        => throw new RuntimeException("Can only encode case classes as directives")
    }
    val name        = nameHint.getOrElse(schemaName)
    val encoder     = fromJsonEncoder(name, jsonEncoder)
    DirectiveEncoder(name, a => encoder.encode(a))
  }

  def gen[A: Schema]: DirectiveEncoder[A] = fromSchema(Schema[A])
}
