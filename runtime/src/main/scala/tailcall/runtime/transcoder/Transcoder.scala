package tailcall.runtime.transcoder

import caliban.parsing.adt.Document
import tailcall.runtime.dsl.json.Config
import tailcall.runtime.internal.TValid
import tailcall.runtime.transcoder.value._

/**
 * A transcoder is a function that takes an A and returns a
 * B, or an error. It can be composed using the >>> operator
 * with other transcoders to create a pipeline. A transcoder
 * between A ~> C can be derived provided there exists a B
 * such that a transcoder from A ~> B exists and a
 * transcoder from B ~> C already exists.
 */
sealed trait Transcoder
    extends Blueprint2Document
    with Config2Blueprint
    with Document2Blueprint
    with Document2Config
    with Document2GraphQLSchema
    with Orc2Blueprint
    with ToDynamicValue
    with ToInputValue
    with ToJsonAST
    with ToResponseValue
    with ToValue

object Transcoder extends Transcoder {
  def toGraphQLSchema(config: Config): TValid[Nothing, String] = toDocument(config).flatMap(toGraphQLSchema(_))

  def toDocument(config: Config): TValid[Nothing, Document] =
    Transcoder.toBlueprint(config).flatMap(Transcoder.toDocument(_))
}
