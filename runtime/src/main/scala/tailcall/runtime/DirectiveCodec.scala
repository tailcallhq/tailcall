package tailcall.runtime

import caliban.parsing.adt.Directive
import tailcall.runtime.DirectiveCodec.{DirectiveDecoder, DirectiveEncoder}
import tailcall.runtime.internal.TValid
import tailcall.runtime.transcoder.Transcoder
import zio.schema.{DynamicValue, Schema}

import scala.collection.immutable.ListMap

/**
 * Allows us to encode decode any scala value as a caliban
 * directive.
 */
final case class DirectiveCodec[A](encoder: DirectiveEncoder[A], decoder: DirectiveDecoder[A]) {
  def decode(directive: Directive): TValid[String, A] = decoder.decode(directive)
  def encode(a: A): TValid[String, Directive]         = encoder.encode(a)
}

object DirectiveCodec {
  trait DirectiveEncoder[A] {
    def encode(a: A): TValid[String, Directive]
  }

  trait DirectiveDecoder[A] {
    def decode(directive: Directive): TValid[String, A]
  }

  object DirectiveEncoder {
    def gen[A](implicit schema: Schema[A]): DirectiveEncoder[A] =
      (a: A) =>
        schema match {
          case record: Schema.Record[_] =>
            val name    = record.id.name
            val dynamic = schema.toDynamic(a)
            for {
              inputValue <- TValid
                .foreachChunk(record.fields)(field => Transcoder.toInputValue(dynamic).map((field.name: String) -> _))
                .map(_.toMap)
            } yield Directive(name, inputValue)
          case _                        => TValid.fail("directives can only be applied to records")
        }
  }

  object DirectiveDecoder {
    def gen[A](implicit schema: Schema[A]): DirectiveDecoder[A] =
      (directive: Directive) =>
        schema match {
          case record: Schema.Record[_] if record.id.name == directive.name =>
            for {
              fields <- TValid.foreach(directive.arguments.toList) { case (name, inputValue) =>
                Transcoder.toDynamicValue(inputValue).map((name: String) -> _)
              }
              a      <- TValid.fromEither(schema.fromDynamic(DynamicValue.Record(record.id, ListMap.from(fields))))
            } yield a

          case _ => TValid.fail("directives can only be applied to records")
        }
  }

  implicit final class EncoderSyntax[A](val self: A) extends AnyVal {
    def toDirective(implicit encoder: DirectiveEncoder[A]): TValid[String, Directive] = encoder.encode(self)
  }

  implicit final class DecoderSyntax(val directive: Directive) extends AnyVal {
    def fromDirective[A](implicit decoder: DirectiveDecoder[A]): TValid[String, A] = decoder.decode(directive)
  }
}
