package com.tailcall.gateway.adt

import zio.Chunk
import zio.json.JsonCodec
import zio.parser._

final case class Route(segments: Chunk[Route.Segment])

object Route {

  sealed trait Segment
  object Segment {
    final case class Literal(value: String) extends Segment
    final case class Param(value: String)   extends Segment
  }

  object syntax {
    val segment = Syntax.alphaNumeric.repeat
      .transform[String](_.asString, s => Chunk.fromIterable(s))

    val param = (Syntax.string("${", ()) ~ segment ~ Syntax.char('}'))
      .transform[Segment.Param](Segment.Param(_), _.value)

    val literal = segment.transform[Segment.Literal](Segment.Literal(_), _.value)

    val segmentChunk = (Syntax.char('/') ~ (literal.widen[Segment] | param.widen[Segment])).repeat

    val route = segmentChunk.transform[Route](Route(_), _.segments)
  }

  implicit val json: JsonCodec[Route] = JsonCodec[String].transformOrFail(
    string =>
      syntax.route.parseString(string) match {
        case Left(_)      => Left(s"Invalid route: ${string}")
        case Right(value) => Right(value)
      },
    syntax.route.asPrinter.printString(_) match {
      // TODO: handle this more gracefully
      case Left(_)      => throw new RuntimeException("Invalid route")
      case Right(value) => value
    },
  )
}
