package tailcall.runtime

import caliban.parsing.adt.Directive
import tailcall.runtime.internal.TValid
import zio.json.{DecoderOps, EncoderOps, JsonDecoder}
import zio.schema.Schema
import zio.schema.annotation.caseName

final case class DirectiveDecoder[A](name: String, decode: Directive => TValid[String, A]) {
  def map[B](ab: A => B): DirectiveDecoder[B] = copy(decode = decode.andThen(_.map(ab)))
}

object DirectiveDecoder {

  def gen[A: Schema]: DirectiveDecoder[A] = fromSchema(Schema[A])

  def fromSchema[A](schema: Schema[A]): DirectiveDecoder[A] = {
    val jsonDecoder = zio.schema.codec.JsonCodec.jsonDecoder(schema)
    val nameHint    = schema.annotations.collectFirst { case caseName(name) => name }
    val schemaName  = schema match {
      case schema: Schema.Enum[_]   => schema.id.name
      case schema: Schema.Record[_] => schema.id.name
      case _                        => throw new RuntimeException("Can only decode case classes as directives")
    }

    val name    = nameHint.getOrElse(schemaName)
    val decoder = fromJsonDecoder(nameHint.getOrElse(name), jsonDecoder)
    DirectiveDecoder(name, decoder.decode(_))
  }

  def fromJsonDecoder[A](name: String, decoder: JsonDecoder[A]): DirectiveDecoder[A] =
    DirectiveDecoder(
      name,
      directive =>
        for {
          _    <-
            if (name != directive.name) TValid.fail(s"Expected directive name to be $name but was ${directive.name}")
            else TValid.succeed(())
          args <- TValid.fromEither(directive.arguments.toJsonAST)
          a    <- TValid.fromEither(args.toJson.fromJson[A](decoder))
        } yield a,
    )

  def apply[A](implicit decoder: DirectiveDecoder[A]): DirectiveDecoder[A] = decoder

  implicit def decoder[A](implicit codec: DirectiveCodec[A]): DirectiveDecoder[A] = codec.decoder
}
