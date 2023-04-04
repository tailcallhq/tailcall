package tailcall.runtime

import caliban.parsing.adt.Directive
import tailcall.runtime.DirectiveCodec.{DirectiveDecoder, DirectiveEncoder}
import tailcall.runtime.internal.TValid
import tailcall.runtime.transcoder.Transcoder
import zio.json.jsonHint
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

  def fromSchema[A](schema: Schema[A]): DirectiveCodec[A] =
    DirectiveCodec(DirectiveEncoder.gen(schema), DirectiveDecoder.gen(schema))

  trait DirectiveEncoder[A] {
    def encode(a: A): TValid[String, Directive]
  }

  trait DirectiveDecoder[A] {
    def decode(directive: Directive): TValid[String, A]
  }

  object DirectiveEncoder {
    def gen[A](implicit schema: Schema[A]): DirectiveEncoder[A] = { (a: A) =>
      schema.toDynamic(a) match {
        case DynamicValue.Record(id, values) =>
          val typeName     = schema.annotations.collectFirst { case jsonHint(name) => name }.getOrElse(id.name)
          val recordSchema = schema.asInstanceOf[Schema.Record[A]]
          val nameMap: Map[String, String] = recordSchema.fields.flatMap { field =>
            field.annotations.collectFirst { case jsonHint(name) => field.name -> name }
          }.toMap
          for {
            map <- TValid.foreach(values.toList) { case (name, dynamicValue) =>
              val fieldName = nameMap.getOrElse(name, name)
              Transcoder.toInputValue(dynamicValue).map(fieldName -> _)
            }.map(_.toMap)
          } yield Directive(typeName, map)
        case _                               => TValid.fail("directives can only be applied to records")
      }
    }
  }

  object DirectiveDecoder {
    def gen[A](implicit schema: Schema[A]): DirectiveDecoder[A] = { (directive: Directive) =>
      schema match {
        case record: Schema.Record[_] =>
          val typeName = schema.annotations.collectFirst { case jsonHint(name) => name }.getOrElse(record.id.name)
          if (directive.name != typeName) TValid
            .fail(s"expected directive name to be $typeName but was ${directive.name}")
          else {
            val nameMap: Map[String, String] = record.fields
              .flatMap(field => field.annotations.collectFirst { case jsonHint(name) => name -> field.name }).toMap
            for {
              fields <- TValid.foreach(directive.arguments.toList) { case (name, inputValue) =>
                val fieldName = nameMap.getOrElse(name, name)
                Transcoder.toDynamicValue(inputValue).map(fieldName -> _)
              }
              _ = pprint.pprintln(fields)
              a      <- TValid.fromEither(schema.fromDynamic(DynamicValue.Record(record.id, ListMap.from(fields))))
            } yield a
          }
        case _                        => TValid.fail("directives can only be applied to records")
      }
    }
  }

  implicit final class EncoderSyntax[A](val self: A) extends AnyVal {
    def toDirective(implicit encoder: DirectiveEncoder[A]): TValid[String, Directive] = encoder.encode(self)
  }

  implicit final class DecoderSyntax(val directive: Directive) extends AnyVal {
    def fromDirective[A](implicit decoder: DirectiveDecoder[A]): TValid[String, A] = decoder.decode(directive)
  }

  implicit def encoder[A](implicit codec: DirectiveCodec[A]): DirectiveEncoder[A] = codec.encoder

  implicit def decoder[A](implicit codec: DirectiveCodec[A]): DirectiveDecoder[A] = codec.decoder
}
