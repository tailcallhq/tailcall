package tailcall.runtime

import caliban.InputValue
import caliban.parsing.adt.Directive
import tailcall.runtime.internal.TValid
import zio.json.{DecoderOps, JsonEncoder}
import zio.schema.Schema
import zio.schema.annotation.caseName

trait DirectiveEncoder[A] {
  self =>
  final def contramap[B](ab: B => A): DirectiveEncoder[B] =
    new DirectiveEncoder[B] {
      override def encode(a: B): TValid[String, Directive] = self.encode(ab(a))
      override def name: String                            = self.name
    }

  def encode(a: A): TValid[String, Directive]
  def name: String

  final def withName(newName: String): DirectiveEncoder[A] =
    new DirectiveEncoder[A] {
      override def encode(a: A): TValid[String, Directive] = self.encode(a)
      override def name: String                            = newName
    }
}

object DirectiveEncoder {
  def gen[A: Schema]: DirectiveEncoder[A] = fromSchema(Schema[A])

  def fromSchema[A](schema: Schema[A]): DirectiveEncoder[A] = {
    val encoder = zio.schema.codec.JsonCodec.jsonEncoder(schema)

    new DirectiveEncoder[A] {
      override def encode(a: A): TValid[String, Directive] = fromJsonEncoder(name, encoder).encode(a)

      override def name: String = {
        val nameHint   = schema.annotations.collectFirst { case caseName(name) => name }
        val schemaName = schema match {
          case schema: Schema.Enum[_]   => schema.id.name
          case schema: Schema.Record[_] => schema.id.name
          case _ => throw new RuntimeException("Can only encode sealed traits and case classes as directives")
        }
        nameHint.getOrElse(schemaName)
      }
    }
  }

  def fromJsonEncoder[A](directiveName: String, encoder: JsonEncoder[A]): DirectiveEncoder[A] =
    new DirectiveEncoder[A] {
      override def encode(a: A): TValid[String, Directive] = {
        for {
          args <- TValid.fromEither(encoder.encodeJson(a).fromJson[Map[String, InputValue]])
        } yield Directive(name, args)
      }

      override def name: String = directiveName
    }

  // FIXME: this should be removed
  def fromJsonListEncoder[A](directiveName: String, encoder: JsonEncoder[A]): DirectiveEncoder[List[A]] =
    new DirectiveEncoder[List[A]] {
      override def encode(a: List[A]): TValid[String, Directive] =
        for {
          args <- TValid.fromEither(JsonEncoder.list(encoder).encodeJson(a).fromJson[InputValue])
        } yield Directive(name, Map("value" -> args))

      override def name: String = directiveName
    }

  def apply[A](implicit encoder: DirectiveEncoder[A]): DirectiveEncoder[A] = encoder

  implicit def encoder[A](implicit codec: DirectiveCodec[A]): DirectiveEncoder[A] = codec.encoder
}
