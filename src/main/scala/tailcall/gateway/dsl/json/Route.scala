package tailcall.gateway.dsl.json

import zio.Chunk
import zio.json.JsonCodec
import zio.parser.Syntax

final case class Route(segments: List[Route.Segment])
object Route {
  sealed trait Segment
  object Segment {
    final case class Literal(value: String) extends Segment
    final case class Param(value: String)   extends Segment
  }

  object syntax {
    val segment = Syntax.alphaNumeric.repeat.transform[String](_.asString, Chunk.fromIterable(_))

    val param = (Syntax.string("${", ()) ~ segment ~ Syntax.char('}'))
      .transform[Segment.Param](Segment.Param, _.value)

    val literal = segment.transform[Segment.Literal](Segment.Literal, _.value)

    val segmentChunk = (Syntax.char('/') ~ (literal.widen[Segment] | param.widen[Segment])).repeat

    val route = segmentChunk
      .transform[Route](chunk => Route(chunk.toList), route => Chunk.from(route.segments))

  }

  def decode(string: String): Either[String, Route] = syntax.route.parseString(string) match {
    case Left(_)      => Left(s"Invalid route: ${string}")
    case Right(value) => Right(value)
  }

  def encode(route: Route): Either[String, String] = syntax.route.asPrinter.printString(route)

  implicit lazy val routeCodec: JsonCodec[Route] = JsonCodec[String].transformOrFail(
    Route.decode,

    // TODO: handle this error more gracefully
    route => Route.encode(route).getOrElse(throw new RuntimeException("Invalid Route"))
  )

}
