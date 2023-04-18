package tailcall.runtime

import caliban.parsing.adt.Directive
import tailcall.runtime.internal.TValid
import zio.json.{DecoderOps, EncoderOps, JsonDecoder}
import zio.schema.Schema
import zio.schema.annotation.caseName

trait DirectiveDecoder[A] {
  def decode(directive: Directive): TValid[String, A]
  final def map[B](ab: A => B): DirectiveDecoder[B] = { (directive: Directive) => decode(directive).map(ab) }
}

object DirectiveDecoder {
  def collect[A](f: Directive => TValid[String, A]): DirectiveDecoder[A] = { (directive: Directive) => f(directive) }

  def gen[A: Schema]: DirectiveDecoder[A] = fromSchema(Schema[A])

  def fromSchema[A](schema: Schema[A]): DirectiveDecoder[A] = {
    val decoder  = zio.schema.codec.JsonCodec.jsonDecoder(schema)
    val nameHint = schema.annotations.collectFirst { case caseName(name) => name }
    DirectiveDecoder { directive =>
      for {
        name <- schema match {
          case schema: Schema.Enum[_]   => TValid.succeed(schema.id.name)
          case schema: Schema.Record[_] => TValid.succeed(schema.id.name)
          case _                        => TValid.fail("Can only decode sealed traits and case classes as directives")
        }
        a    <- fromJsonDecoder(nameHint.getOrElse(name), decoder).decode(directive)
      } yield a
    }
  }

  def fromJsonDecoder[A](name: String, decoder: JsonDecoder[A]): DirectiveDecoder[A] =
    DirectiveDecoder { directive =>
      for {
        _    <-
          if (name != directive.name) TValid.fail(s"Expected directive name to be $name but was ${directive.name}")
          else TValid.succeed(())
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

  implicit def decoder[A](implicit codec: DirectiveCodec[A]): DirectiveDecoder[A] = codec.decoder
}
