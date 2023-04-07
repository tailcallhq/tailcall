package tailcall.runtime

import caliban.InputValue
import caliban.parsing.adt.Directive
import tailcall.runtime.internal.TValid
import zio.json.{DecoderOps, JsonEncoder}
import zio.schema.Schema
import zio.schema.annotation.caseName

trait DirectiveEncoder[A] {
  final def contramap[B](ab: B => A): DirectiveEncoder[B] = { (b: B) => encode(ab(b)) }

  def encode(a: A): TValid[String, Directive]

  final def withName(name: String): DirectiveEncoder[A] = { (a: A) =>
    for { directive <- encode(a) } yield directive.copy(name = name)
  }
}

object DirectiveEncoder {

  def collect[A](f: A => TValid[String, Directive]): DirectiveEncoder[A] = { (a: A) => f(a) }

  def gen[A: Schema]: DirectiveEncoder[A] = fromSchema(Schema[A])

  def fromSchema[A](schema: Schema[A]): DirectiveEncoder[A] = {
    val encoder  = zio.schema.codec.JsonCodec.jsonEncoder(schema)
    val nameHint = schema.annotations.collectFirst { case caseName(name) => name }
    DirectiveEncoder { a: A =>
      for {
        name      <- schema match {
          case schema: Schema.Enum[_]   => TValid.succeed(schema.id.name)
          case schema: Schema.Record[_] => TValid.succeed(schema.id.name)
          case _                        => TValid.fail("Can only encode sealed traits and case classes as directives")
        }
        directive <- fromJsonEncoder(nameHint.getOrElse(name), encoder).encode(a)
      } yield directive
    }
  }

  def fromJsonEncoder[A](name: String, encoder: JsonEncoder[A]): DirectiveEncoder[A] =
    DirectiveEncoder { a: A =>
      for {
        args <- TValid.fromEither(encoder.encodeJson(a).fromJson[Map[String, InputValue]])
      } yield Directive(name, args)
    }

  def apply[A](implicit encoder: DirectiveEncoder[A]): DirectiveEncoder[A] = encoder

}
