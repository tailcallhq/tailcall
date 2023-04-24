package tailcall.runtime

import caliban.parsing.adt.Directive
import tailcall.runtime.internal.TValid
import tailcall.runtime.model.Blueprint
import tailcall.runtime.transcoder.Transcoder
import zio.json.JsonCodec
import zio.schema.Schema

/**
 * Allows us to encode decode any scala value as a caliban
 * directive.
 */
final case class DirectiveCodec[A](encoder: DirectiveEncoder[A], decoder: DirectiveDecoder[A]) {
  def decode(directive: Directive): TValid[String, A]       = decoder.decode(directive)
  def encode(a: A): TValid[String, Directive]               = encoder.encode(a)
  def transform[B](f: A => B, g: B => A): DirectiveCodec[B] = DirectiveCodec(encoder.contramap(g), decoder.map(f))
  def withName(name: String): DirectiveCodec[A]             = DirectiveCodec(encoder.withName(name), decoder)
  def name: String                                          = encoder.name
}

object DirectiveCodec {

  def fromJsonCodec[A](name: String, codec: JsonCodec[A]): DirectiveCodec[A] =
    DirectiveCodec(
      DirectiveEncoder.fromJsonEncoder(name, codec.encoder),
      DirectiveDecoder.fromJsonDecoder(name, codec.decoder),
    )

  def gen[A: Schema]: DirectiveCodec[A] = fromSchema(Schema[A])

  def fromSchema[A](schema: Schema[A]): DirectiveCodec[A] =
    DirectiveCodec(DirectiveEncoder.fromSchema(schema), DirectiveDecoder.fromSchema(schema))

  implicit final class DecoderSyntax(val directive: Directive) extends AnyVal {
    def fromDirective[A](implicit decoder: DirectiveDecoder[A]): TValid[String, A] = decoder.decode(directive)
  }

  implicit final class EncoderSyntax[A](val self: A) extends AnyVal {
    def toBlueprintDirective(implicit encoder: DirectiveEncoder[A]): TValid[String, Blueprint.Directive] = {
      for {
        directive <- toDirective
        args      <- TValid.foreach(directive.arguments.toList) { case (key, value) =>
          Transcoder.toDynamicValue(value).map(key -> _)
        }
      } yield Blueprint.Directive(directive.name, args.toMap)
    }

    def toDirective(implicit encoder: DirectiveEncoder[A]): TValid[String, Directive] = encoder.encode(self)
  }

}
