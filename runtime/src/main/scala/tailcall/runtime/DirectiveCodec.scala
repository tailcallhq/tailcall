package tailcall.runtime

import caliban.InputValue
import caliban.parsing.adt.Directive
import tailcall.runtime.DirectiveCodec.{DirectiveDecoder, DirectiveEncoder}
import tailcall.runtime.internal.TValid
import tailcall.runtime.model.Blueprint
import tailcall.runtime.transcoder.Transcoder
import zio.json.{DecoderOps, EncoderOps}
import zio.schema.Schema
import zio.schema.annotation.caseName

/**
 * Allows us to encode decode any scala value as a caliban
 * directive.
 */
final case class DirectiveCodec[A](encoder: DirectiveEncoder[A], decoder: DirectiveDecoder[A]) {
  def decode(directive: Directive): TValid[String, A]       = decoder.decode(directive)
  def encode(a: A): TValid[String, Directive]               = encoder.encode(a)
  def transform[B](f: A => B, g: B => A): DirectiveCodec[B] = DirectiveCodec(encoder.contramap(g), decoder.map(f))
}

object DirectiveCodec {
  def apply[A](from: A => TValid[String, Directive], to: Directive => TValid[String, A]): DirectiveCodec[A] =
    DirectiveCodec(DirectiveEncoder.collect(from), DirectiveDecoder.collect(to))

  def fromSchema[A](schema: Schema[A]): DirectiveCodec[A] =
    DirectiveCodec(DirectiveEncoder.fromSchema(schema), DirectiveDecoder.fromSchema(schema))

  trait DirectiveEncoder[A] {
    final def contramap[B](ab: B => A): DirectiveEncoder[B] = { (b: B) => encode(ab(b)) }

    def encode(a: A): TValid[String, Directive]
  }

  trait DirectiveDecoder[A] {
    def decode(directive: Directive): TValid[String, A]
    final def map[B](ab: A => B): DirectiveDecoder[B] = { (directive: Directive) => decode(directive).map(ab) }
  }

  object DirectiveEncoder {
    def fromSchema[A](schema: Schema[A]): DirectiveEncoder[A] =
      DirectiveEncoder { a: A =>
        val encoder  = zio.schema.codec.JsonCodec.jsonEncoder(schema)
        val nameHint = schema.annotations.collectFirst { case caseName(name) => name }
        for {
          name <- schema match {
            case schema: Schema.Enum[_]   => TValid.succeed(schema.id.name)
            case schema: Schema.Record[_] => TValid.succeed(schema.id.name)
            case _                        => TValid.fail("Can only encode sealed traits and case classes as directives")
          }
          args <- TValid.fromEither(encoder.encodeJson(a).fromJson[Map[String, InputValue]])
        } yield Directive(nameHint.getOrElse(name), args)

      }

    def collect[A](f: A => TValid[String, Directive]): DirectiveEncoder[A]   = { (a: A) => f(a) }
    def apply[A](implicit encoder: DirectiveEncoder[A]): DirectiveEncoder[A] = encoder
  }

  object DirectiveDecoder {
    def fromSchema[A](schema: Schema[A]): DirectiveDecoder[A] =
      DirectiveDecoder { directive =>
        val decoder = zio.schema.codec.JsonCodec.jsonDecoder(schema)
        for {
          args <- TValid.fromEither(directive.arguments.toJsonAST)
          a    <- TValid.fromEither(args.toJson.fromJson[A](decoder))
        } yield a
      }

    def collect[A](f: Directive => TValid[String, A]): DirectiveDecoder[A] = { (directive: Directive) => f(directive) }
    def apply[A](implicit decoder: DirectiveDecoder[A]): DirectiveDecoder[A] = decoder
  }

  implicit final class EncoderSyntax[A](val self: A) extends AnyVal {
    def toDirective(implicit encoder: DirectiveEncoder[A]): TValid[String, Directive] = encoder.encode(self)
    def toBlueprintDirective(implicit encoder: DirectiveEncoder[A]): TValid[String, Blueprint.Directive] = {
      for {
        directive <- toDirective
        args      <- TValid.foreach(directive.arguments.toList) { case (key, value) =>
          Transcoder.toDynamicValue(value).map(key -> _)
        }
      } yield Blueprint.Directive(directive.name, args.toMap)
    }
  }

  implicit final class DecoderSyntax(val directive: Directive) extends AnyVal {
    def fromDirective[A](implicit decoder: DirectiveDecoder[A]): TValid[String, A] = decoder.decode(directive)
  }

  implicit def encoder[A](implicit codec: DirectiveCodec[A]): DirectiveEncoder[A] = codec.encoder

  implicit def decoder[A](implicit codec: DirectiveCodec[A]): DirectiveDecoder[A] = codec.decoder
}
