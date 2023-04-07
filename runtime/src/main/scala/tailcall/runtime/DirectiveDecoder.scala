package tailcall.runtime

import caliban.parsing.adt.Directive
import tailcall.runtime.internal.TValid
import zio.json.{DecoderOps, EncoderOps, JsonDecoder}
import zio.schema.Schema

trait DirectiveDecoder[A] {
  def decode(directive: Directive): TValid[String, A]
  final def map[B](ab: A => B): DirectiveDecoder[B] = { (directive: Directive) => decode(directive).map(ab) }
}

object DirectiveDecoder {
  def collect[A](f: Directive => TValid[String, A]): DirectiveDecoder[A] = { (directive: Directive) => f(directive) }

  def gen[A: Schema]: DirectiveDecoder[A] = fromSchema(Schema[A])

  def fromSchema[A](schema: Schema[A]): DirectiveDecoder[A] = {
    val decoder = zio.schema.codec.JsonCodec.jsonDecoder(schema)
    fromJsonDecoder(decoder)
  }

  def fromJsonDecoder[A](decoder: JsonDecoder[A]): DirectiveDecoder[A] =
    DirectiveDecoder { directive =>
      for {
        args <- TValid.fromEither(directive.arguments.toJsonAST)
        a    <- TValid.fromEither(args.toJson.fromJson[A](decoder))
      } yield a
    }

  def fromJsonListDecoder[A](decoder: JsonDecoder[A]): DirectiveDecoder[List[A]] =
    DirectiveDecoder { directive =>
      for {
        inputValue <- directive.arguments.get("value") match {
          case Some(inputValue) => TValid.succeed(inputValue)
          case None             => TValid.fail(s"key `value` was not found in directive ${directive.name}")
        }
        a          <- TValid.fromEither(inputValue.toJson.fromJson[List[A]](JsonDecoder.list(decoder)))
      } yield a
    }

  def apply[A](implicit decoder: DirectiveDecoder[A]): DirectiveDecoder[A] = decoder

}
